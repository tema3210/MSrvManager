use std::{fs::File, io::{ErrorKind, Read, Seek, SeekFrom, Write}, path::Path, process::Child, thread};

use crate::*;

use anyhow::anyhow;
use std::process::Command;

#[derive(Debug)]
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
    pub instance_state: InstanceState,
    /// should point at directory where Instance is located
    pub place: Arc<Path>,

    manifest: std::fs::File,
    run_command: std::process::Command,

    process: Option<std::process::Child>
}

pub const MANIFEST_NAME: &str = "msrvDesc.json";

pub const COMMAND_FILE_NAME: &str = "run.command";

impl Instance {
    /// should create dir and apropriate manifest files there
    pub fn prepare<P: AsRef<Path>>(at: P) -> anyhow::Result<()> {
        log::info!("preparing dir for server at {:?}",at.as_ref());
        std::fs::create_dir(&at)?;

        std::fs::File::create_new(at.as_ref().join(instance::MANIFEST_NAME))?;
        std::fs::File::create_new(at.as_ref().join(instance::COMMAND_FILE_NAME))?;

        Ok(())
    }

    fn open_manifest(at: &Path) -> Result<File, std::io::Error> {
        File::options()
            .write(true)    
            .read(true)
            .open(at.join(MANIFEST_NAME))
    }

    fn read_run_command(at: &Path) -> anyhow::Result<Command> {
        let mut cmd_file = File::open(at.join(COMMAND_FILE_NAME))?;
        let mut run_command = String::new();

        cmd_file.read_to_string(&mut run_command)?;

        let mut run_command = Command::new(run_command);
        run_command.current_dir(at);

        Ok(run_command)
    }

    pub fn create(place: Arc<Path>, desc: model::InstanceDescriptor,state: InstanceState) -> anyhow::Result<Self> {
        if !place.is_dir() {
            return Err(anyhow!("create should be called on dir"));
        }

        let mut manifest = Self::open_manifest(&*place)?;

        desc.to_file(&mut manifest)?;

        let run_command = Self::read_run_command(&*place)?;

        Ok(Instance {
            desc,
            instance_state: state,
            process: None,
            place,
            manifest,
            run_command
        })
    }

    pub fn load(place: Arc<Path>) -> anyhow::Result<Self> {

        log::info!("Loading server instance at {:?}",&*place);

        if !place.is_dir() {
            return Err(anyhow!("load should be called on dir"));
        }

        let mut manifest = Self::open_manifest(&*place)?;
        let desc: model::InstanceDescriptor = model::InstanceDescriptor::from_file(&mut manifest)?;

        let mut command_file = std::fs::File::open((&*place).join(COMMAND_FILE_NAME))?;

        let mut run_command = String::new();

        command_file.read_to_string(&mut run_command)?;

        let mut run_command = Command::new(run_command);
        run_command.current_dir(&*place);

        Ok(Self {desc, place, manifest, run_command, process: None, instance_state: InstanceState::Normal})
    }

    // we don't have anything to do reasonably in case of failure
    pub fn flush(&mut self) {
        let _ = self.manifest.seek(SeekFrom::Start(0));
        let _ = self.manifest.set_len(0);
        let _ = serde_json::to_writer(&mut self.manifest, &self.desc);
        let _ = self.manifest.flush();
    }

    pub fn hb(&mut self) {
        
        if !matches!(self.instance_state,InstanceState::Normal) {
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
                        let memory = (memory / (1024 * 1024)) as f64;
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
        if !matches!(self.instance_state,InstanceState::Normal) {
            return
        }
        match self.process {
            Some(_) => return,
            None => {
                if let Ok(ch) = self.run_command.spawn() {
                    self.process = Some(ch)
                };
                self.hb()
            }
        };
    }

    fn stop_inner(ch: &mut Child, name: Arc<Path>) {
        if let Some(pipe) =  &mut ch.stdin {
            loop {
                match pipe.write(b"stop\n") {
                    Ok(0) => break, // dead
                    Ok(_) => {
                        std::thread::sleep(std::time::Duration::from_secs(5));
                        continue;
                    },
                    Err(e) if e.kind() == ErrorKind::Interrupted => {
                        continue
                    },
                    Err(e) => {
                        log::error!("cannot stop {:?} due to {} - killing",name,e);
                        let _ = ch.kill();
                        break
                    }
                }
            }
        };
    }


    pub fn stop_async(&mut self, addr: native::Service) {
        if matches!(self.instance_state,InstanceState::Normal) {
            return
        }

        if let Some(mut ch) = self.process.take() {
            self.instance_state = InstanceState::Stopping;
            let name = self.place.clone();
            thread::spawn(move || {
                Self::stop_inner(&mut ch, Arc::clone(&name));
                addr.do_send(messages::InstanceStopped(name));
            });
            
        };
        
    }

    pub fn finish_stop(&mut self) {
        self.desc.state = model::ServerState::Stopped;
        self.desc.memory = None;
        self.flush();
        self.instance_state = InstanceState::Normal;
    }

    /// blocks current thread
    pub fn stop(&mut self) {
        if !matches!(self.instance_state,InstanceState::Normal) {
            return
        }

        if let Some(mut ch) = self.process.take() {
            Self::stop_inner(&mut ch, self.place.clone())
        }

        self.finish_stop()
    }

    pub fn kill(&mut self) {
        match &mut self.process {
            Some(ch) => {
                // here we can only have a zombie process failure, which is not a failure as such for us
                let _ = ch.kill();
                self.desc.state = model::ServerState::Stopped;
                self.desc.memory = None
            },
            None => {}
        }
    }
}