use std::{fs::File, io::{ErrorKind, Seek, SeekFrom, Write}, path::Path, process::Child, thread};

use wait_timeout::ChildExt;

use crate::*;

use anyhow::anyhow;
use std::process::Command;

#[derive(Debug,Clone,Copy)]
pub enum InstanceState {
    Normal,
    Downloading,
    Stopping
}


/// The descriptor of a server
/// at the dir pointed by `place`
/// there should be `msrvDesc.json` file
/// and `run.command` file
#[derive(Debug)]
pub struct Instance {
    pub desc: model::InstanceDescriptor,
    pub state: InstanceState,
    /// should point at directory where Instance is located
    pub place: Arc<Path>,

    manifest: std::fs::File,

    process: Option<std::process::Child>
}

pub const MANIFEST_NAME: &str = "msrvDesc.json";

const PATCH_SH_PATH: &str = "/app/patch.sh";

pub const SERVER_PROPERTIES_FILE: &str = "server.properties";

impl Instance {
    /// should create dir and apropriate manifest files there
    pub fn prepare<P: AsRef<Path>>(at: P) -> anyhow::Result<()> {
        log::trace!("preparing dir for server at {:?}",at.as_ref());
        std::fs::create_dir(&at)?;

        std::fs::File::create_new(at.as_ref().join(instance::MANIFEST_NAME))?;

        Ok(())
    }

    fn open_manifest(at: &Path) -> Result<File, std::io::Error> {
        File::options()
            .write(true)    
            .read(true)
            .open(at.join(MANIFEST_NAME))
    }

    pub fn patch_server_props(&self) -> anyhow::Result<()> {
        let output = Command::new("sh")
            .arg(PATCH_SH_PATH)
            .env("MPORT", self.desc.port.to_string())
            .env("MRCON", self.desc.rcon.to_string())
            .env("MAXMEMORY", format!("{}G", self.desc.max_memory))
            .env("PROPERTIES_FILE", self.place.join(SERVER_PROPERTIES_FILE))
            .output()?;

        if !output.status.success() {
            Err(anyhow!(
                "patch_server_props script failed with status: {}",
                output.status
            ))
        } else {
            log::trace!("patch_server_props script executed successfully");
            Ok(())
        }
    }

    pub fn create(place: Arc<Path>, desc: model::InstanceDescriptor,state: InstanceState) -> anyhow::Result<Self> {
        if !place.is_dir() {
            return Err(anyhow!("create should be called on dir"));
        }

        let mut manifest = Self::open_manifest(&*place)?;

        desc.to_file(&mut manifest)?;

        Ok(Instance {
            desc,
            state,
            process: None,
            place,
            manifest,
        })
    }

    pub fn load(place: Arc<Path>) -> anyhow::Result<Self> {

        log::info!("Loading server instance at {:?}",&*place);

        if !place.is_dir() {
            return Err(anyhow!("load should be called on dir"));
        }

        let mut manifest = Self::open_manifest(&*place)?;
        let desc: model::InstanceDescriptor = model::InstanceDescriptor::from_file(&mut manifest)?;

        Ok(Self {desc, place, manifest, process: None, state: InstanceState::Normal})
    }

    // we don't have anything to do reasonably in case of failure
    pub fn flush(&mut self) {
        let _ = self.manifest.seek(SeekFrom::Start(0));
        let _ = self.manifest.set_len(0);
        let _ = serde_json::to_writer(&mut self.manifest, &self.desc);
        let _ = self.manifest.flush();
    }

    pub fn hb(&mut self) {
        
        if !matches!(self.state,InstanceState::Normal) {
            return
        }

        match &mut self.process {
            Some(ch) => {
                if let Ok(process) = procfs::process::Process::new(ch.id().try_into().unwrap()) {
                    let Ok(status) = process.status() else {
                        return;
                    };
                    // memory in KB
                    if let Some(memory) = status.vmrss {
                        let memory = ( memory / 1024 ) as f64 / 1024.0;
                        self.desc.memory = Some(memory)
                    }
                } else {
                    if self.desc.state == model::ServerState::Running {
                        self.desc.state = model::ServerState::Crashed
                    }
                }
            }, 
            None => {}
        }
        self.flush()
    }

    pub fn start(&mut self) {
        if !matches!(self.state,InstanceState::Normal) {
            return
        }
        match self.process {
            Some(_) => {
                log::error!("start called on running instance");
                return
            },
            None => {
                log::info!("starting server {:?}", &self.place);

                match self.patch_server_props() {
                    Ok(_) => {},
                    Err(e) => {
                        log::error!("cannot patch server properties due to {}",e);
                        self.desc.state = model::ServerState::Crashed;
                        return
                    }
                }

                let mut cmd = Command::new("java");
                cmd.current_dir(self.place.join(self.desc.server_jar.parent().unwrap()))
                    .arg(format!("-Xmx{}M", (self.desc.max_memory * 1024.0) as u64))
                    .args(self.desc.java_args.iter())
                    .arg("-jar")
                    .arg(self.desc.server_jar.as_os_str())
                    .arg("--nogui")
                ;

                match cmd.spawn() {
                    Ok(ch) => {
                        self.process = Some(ch);
                        self.desc.state = model::ServerState::Running;
                        self.hb()
                    },
                    Err(e) => {
                        log::error!("cannot start server due to {} - used {:?}",e,&cmd);
                        self.desc.state = model::ServerState::Crashed;
                    }
                }
            }
        };
    }

    
    fn stop_inner(mut ch: Child, name: impl AsRef<Path>) {
        log::info!("stopping server {:?} factually", &name.as_ref());
        const STOP_CMD: &[u8] = b"stop\n";
        if let Some(pipe) = &mut ch.stdin {
            let mut written = 0;
            while written < STOP_CMD.len() {
                match pipe.write(&STOP_CMD[written..]) {
                    Ok(0) => break, // dead or written
                    Ok(n) => {
                        written += n;
                    },
                    Err(e) if e.kind() == ErrorKind::Interrupted => {
                        continue;
                    },
                    Err(e) => {
                        log::error!("cannot stop {:?} due to {} - killing", name.as_ref(), e);
                        break;
                    }
                }
            };

            match ch.wait_timeout(std::time::Duration::from_secs(10)) {
                Ok(Some(status)) => {
                    log::info!("server {:?} stopped with status {:?}", name.as_ref(), status);
                    Instance::dispose(ch);
                }
                Ok(None) => {
                    log::warn!("server {:?} did not stop in time, killing", name.as_ref());
                    let _ = ch.kill();
                    Instance::dispose(ch);
                }
                Err(e) => {
                    log::error!("error while waiting for server {:?} to stop: {}", name.as_ref(), e);
                    let _ = ch.kill();
                    Instance::dispose(ch);
                }
            }
        } else {
            log::error!("stdin pipe is not available for {:?}", name.as_ref());
            let _ = ch.kill();
            Instance::dispose(ch);
        }
    }

    pub fn finish_stop(&mut self) {
        self.desc.state = model::ServerState::Stopped;
        self.desc.memory = None;
        self.flush();
        self.state = InstanceState::Normal;
    }

    pub fn stop_async(&mut self, addr: native::Service) {
        log::info!("stopping server async {:?}", &self.place);

        if !matches!(self.state,InstanceState::Normal) {
            return
        }

        if let Some(ch) = self.process.take() {
            self.state = InstanceState::Stopping;
            let name = self.place.clone();
            thread::spawn(move || {
                Self::stop_inner(ch, &name);
                addr.do_send(messages::InstanceStopped(name));
            });
            
        };
        
    }


    /// blocks current thread
    pub fn stop(&mut self) {
        log::info!("stopping server {:?}", &self.place);

        if !matches!(self.state,InstanceState::Normal) {
            return
        }     

        if let Some(ch) = self.process.take() {
            Self::stop_inner(ch, self.place.clone())
        }

        self.finish_stop()
    }

    pub fn kill(&mut self) {
        match self.process.take() {
            Some(mut ch) => {
                let _ = ch.kill();
                Instance::dispose(ch);

                self.desc.state = model::ServerState::Stopped;
                self.desc.memory = None
            },
            None => {}
        }
    }
    
    /// it only disposed process, not kills its
    fn dispose(mut child: Child) {
        std::thread::spawn(move || {
            let r = child.wait();
            match r {
                Ok(status) => {
                    log::info!("child with pid {} died with status: {:?}", child.id(), status);
                }
                Err(e) => {
                    log::error!("error while waiting for child to die: {}", e);
                }
            }
        });
    }
}