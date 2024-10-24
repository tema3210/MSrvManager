use std::io::ErrorKind;
use std::{io::Write, path::Path, process::Child};

use actix::prelude::*;
use async_graphql::UploadValue;
use wait_timeout::ChildExt;

use crate::*;

use crate::messages::instance_messages;

use anyhow::anyhow;
use std::process::Command;

pub struct InstanceData {
    pub desc: model::InstanceDescriptor,
    manifest: std::fs::File,
}

pub enum InstanceState {
    Running {
        child: Child,
        rcon: rcon::Rcon,
        data: InstanceData
    },
    Starting {
        child: Child,
        data: InstanceData
    },
    Crashed {
        data: InstanceData
    },
    Stopped {
        data: InstanceData
    },
    Downloading {
        desc: model::InstanceDescriptor,
        // setup_cmd: Option<Command>,
        payload: UploadValue
    },
    /// this state is blank, used for transactional operations
    Swap
}

pub struct InstanceEnv {
    pub servers: Addr<native::Servers>,
    pub timeout: std::time::Duration,
    pub password: String
}

/// The descriptor of a server
/// at the dir pointed by `place`
/// there should be `msrvDesc.json` file, manifest field
pub struct Instance {
    /// should point at directory where Instance is or should be located located
    place: Arc<Path>,
    env: InstanceEnv,

    state: InstanceState,
}

impl Instance {
    pub fn place(&self) -> &Path {
        &*self.place
    }

    pub fn name(&self) -> String {
        self.place.file_name().unwrap().to_string_lossy().to_string()
    }

    pub fn state(&self) -> model::InstanceState {
        match self.state {
            InstanceState::Running { .. } => model::InstanceState::Running,
            InstanceState::Starting { .. } => model::InstanceState::Starting,
            InstanceState::Crashed { .. } => model::InstanceState::Crashed,
            InstanceState::Stopped { .. } => model::InstanceState::Stopped,
            InstanceState::Downloading { .. } => model::InstanceState::Downloading,
            InstanceState::Swap => model::InstanceState::Busy
        }
    }

    pub fn desc(&self) -> Option<&model::InstanceDescriptor> {
        match &self.state {
            InstanceState::Running { data, .. } |
            InstanceState::Starting { data, .. } |
            InstanceState::Crashed { data, .. } |
            InstanceState::Stopped { data, .. } => Some(&data.desc),
            _ => None
        }
    }
}

#[derive(Debug)]
pub enum LoadError {
    PathIsNotDir,
    NoManifest(std::io::Error),
    BadManifest(model::IDError)
}

//ctors
impl Instance {
    pub fn create(
        at: Arc<Path>,
        desc: model::InstanceDescriptor,
        // cmd: Option<String>,
        payload: UploadValue,
        env: InstanceEnv,
    ) -> Self {
        let state = InstanceState::Downloading {
            desc,
            // setup_cmd: cmd.map(utils::make_command),
            payload
        };

        Self {place: at, state, env}
    }

    pub fn load(place: Arc<Path>, env: InstanceEnv ) -> Result<(Self,model::Ports),LoadError> {

        log::info!("Loading server instance at {:?}",&*place);

        if !place.is_dir() {
            return Err(LoadError::PathIsNotDir);
        }

        let mut manifest = utils::open_manifest(&*place).map_err(|e| LoadError::NoManifest(e))?;
        let desc: model::InstanceDescriptor = model::InstanceDescriptor::from_file(&mut manifest).map_err(|e| LoadError::BadManifest(e))?;

        let ports = desc.ports;

        Ok((
            Self {
                place, 
                state: InstanceState::Stopped {
                    data: InstanceData {
                        desc,
                        manifest
                    }
                },
                env
            },
            ports    
        ))
    }
}

impl Actor for Instance {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {

        let state = match &mut self.state {
            // we whould start downloading
            state @ InstanceState::Downloading { .. } => std::mem::replace(state, InstanceState::Swap),
            // it's fine
            InstanceState::Stopped { .. } | InstanceState::Crashed { .. } => return,
            _ => {
                log::error!("Instance started in bad state: {:?}", &self.place);
                ctx.stop();
                return
            }
        };
        
        match state {
            InstanceState::Downloading {  desc,mut payload, } => { // setup_cmd
                if let Err(e) = utils::initialize_server_directory(&self.place,|| {
                    Ok(utils::unpack_at(&self.place, &mut payload)?)
                }) {
                    log::error!("cannot initialize server directory: {:?}",e);
                    ctx.stop();
                    return;
                };

                let mut data = InstanceData {
                    desc,
                    manifest: utils::open_manifest(&self.place).unwrap()
                };

                data.desc.flush(&mut data.manifest).unwrap();

                self.state = InstanceState::Stopped { data };

                // if let Some(mut cmd) = setup_cmd {
                //     let output = cmd.output().unwrap();
                //     if !output.status.success() {
                //         log::error!("setup command failed with status: {}", output.status);
                //         drop(data);
                //         self.env.servers.do_send(native_messages::Nuke { who: Arc::clone(&self.place) });

                //         ctx.stop();
                //         return;
                //     } else {
                //         log::trace!("setup command executed successfully");
                //         self.state = InstanceState::Stopped { data };
                //     }
                // } else {
                //     self.state = InstanceState::Stopped { data };
                // }
            },
            InstanceState::Crashed { .. } | InstanceState::Stopped { .. } => return,
            _ => unreachable!()
        }

        log::info!("Instance started: {:?}", &self.place);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        log::info!("Instance stopped: {:?}", &self.place);
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        log::info!("Instance stopping: {:?}", &self.place);
        Running::Stop
    }
}

pub const MANIFEST_NAME: &str = "msrvDesc.json";

pub const PATCH_SH_PATH: &str = "/app/patch.sh";

pub const SERVER_PROPERTIES_FILE: &str = "server.properties";

impl Instance {

    pub fn run(
        at: Arc<Path>,
        desc: model::InstanceDescriptor,
        data: InstanceData
    ) -> anyhow::Result<InstanceState> {
        let password = std::env::var("PASSWORD")
            .expect("no password specified");

        utils::patch_server_props(
            at.as_ref(),
            desc.ports.port,
            desc.ports.rcon,
            desc.max_memory as usize,
            &password
        )?;

        let mut cmd = Command::new("java");

        let classpath = at.as_ref().join("libraries");

        let classpath = utils::generate_classpath(classpath)?;

        cmd.current_dir(
            at.as_ref()
                // .join(desc.server_jar.parent().unwrap())
        )
            .arg(format!("-Xmx{}M", (desc.max_memory * 1024.0) as u64))
            .args(["-DlegacyClassPath".into(), classpath])
            .args(desc.java_args.iter())
            // .arg("-jar")
            // .arg(desc.server_jar.as_os_str())
            .arg("--nogui")
            .stdin(std::process::Stdio::piped())
        ;

        log::info!("starting process for: {:?}", &at);

        return Ok(InstanceState::Starting {
            child: cmd.spawn()?,
            data
        });
    }

    fn stop_inner(mut ch: Child, name: impl AsRef<Path>,timeout: std::time::Duration) {
        log::info!("stopping server {:?}", &name.as_ref());
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

            match ch.wait_timeout(timeout) {
                Ok(Some(status)) => {
                    log::info!("server {:?} stopped with status {:?}", name.as_ref(), status);
                    utils::dispose(ch);
                }
                Ok(None) => {
                    log::warn!("server {:?} did not stop in time, killing", name.as_ref());
                    let _ = ch.kill();
                    utils::dispose(ch);
                }
                Err(e) => {
                    log::error!("error while waiting for server {:?} to stop: {}", name.as_ref(), e);
                    let _ = ch.kill();
                    utils::dispose(ch);
                }
            }
        } else {
            log::error!("stdin pipe is not available for {:?} killing", name.as_ref());
            let _ = ch.kill();
            utils::dispose(ch);
        }
    }
    
}

impl Handler<messages::Tick> for Instance {
    type Result = ();

    fn handle(&mut self, _: messages::Tick, _ctx: &mut Self::Context) -> Self::Result {

        let (child, data) = match &mut self.state {
            InstanceState::Starting { child, data } | 
            InstanceState::Running { child, data, .. } => (child,data),
            InstanceState::Crashed { data } |
            InstanceState::Stopped { data } => {
                data.desc.memory = None;
                let _ =  data.desc.flush(&mut data.manifest);
                return
            },
            InstanceState::Downloading { .. } | InstanceState::Swap => return,
            
        };

        if let Ok(process) = procfs::process::Process::new(child.id().try_into().unwrap()) {
            let Ok(status) = process.status() else {
                return;
            };
            // memory in KB
            if let Some(memory) = status.vmrss {
                let memory = ( memory / 1024 ) as f64 / 1024.0;
                data.desc.memory = Some(memory)
            }

            return;
        }
        log::error!("cannot get process info for {:?}", &self.place);

        match std::mem::replace(&mut self.state, InstanceState::Swap) {
            InstanceState::Starting { data, .. } |
            InstanceState::Running { data, ..  } => {
                self.state = InstanceState::Crashed { data };
            },
            _ => unreachable!()
        }

    }
}

impl Handler<instance_messages::Kill> for Instance {
    type Result = anyhow::Result<()>;

    fn handle(&mut self, _msg: instance_messages::Kill, _ctx: &mut Self::Context) -> Self::Result {
        let (mut child,data) = match std::mem::replace(&mut self.state, InstanceState::Swap) {
            InstanceState::Running { child, data, .. } => (child,data),
            InstanceState::Starting { child, data } => (child,data),
            old => {
                self.state = old;
                log::info!("server {:?} is already stopped", &self.place);
                return Ok(())
            }
        };
        let _ = child.kill();
        utils::dispose(child);

        self.state = InstanceState::Crashed { data };
        Ok(())
    }
}

impl<O,F> Handler<instance_messages::Instance<O,F>> for Instance 
    where
        O: Send + 'static,
        F: Sync + Send + Fn(&Instance) -> Option<O> + 'static
{
    type Result = Option<O>;

    fn handle(&mut self, msg: instance_messages::Instance<O,F>, _ctx: &mut Self::Context) -> Self::Result {
        (msg.f)(&self)
    }
}


impl Handler<rcon::RconMessage> for Instance {
    type Result = anyhow::Result<()>;

    fn handle(&mut self, msg: rcon::RconMessage, _ctx: &mut Self::Context) -> Self::Result {
        match &mut self.state {
            InstanceState::Running { rcon, .. } => {
                Ok(rcon.send(msg.cmd)?)
            },
            _ => {
                log::error!("rcon is not available for {:?}", &self.place);
                Err(anyhow!("rcon is not available for {:?}", &self.place))
            }
        }
    }
}

impl Handler<instance_messages::SwitchServer> for Instance {
    type Result = anyhow::Result<()>;

    fn handle(&mut self, msg: instance_messages::SwitchServer, ctx: &mut Self::Context) -> Self::Result {
        match std::mem::replace(&mut self.state, InstanceState::Swap) {
            InstanceState::Running { child, data, .. } => {
                if msg.should_run {
                    log::info!("server {:?} is already running", &self.place);
                    return Ok(())
                }

                log::info!("stopping server {:?}", &self.place);

                Instance::stop_inner(child, &self.place,self.env.timeout);

                self.state = InstanceState::Stopped { data };

                Ok(())
            },
            InstanceState::Crashed { data } | InstanceState::Stopped { data } => {
                if msg.should_run {
                    let timeout = self.env.timeout;

                    let rcon = data.desc.ports.rcon;

                    let password = self.env.password.clone();

                    let this = ctx.address();

                    log::info!("starting server {:?}", &self.place);

                    let next_state = Self::run(
                        Arc::clone(&self.place),
                        data.desc.clone(),
                        data
                    )?;

                    self.state = next_state;

                    tokio::spawn(async move {

                        tokio::time::sleep(timeout).await;

                        let rcon = rcon::Rcon::new(
                            rcon,
                            password
                        ).await;

                        match rcon {
                            Ok(rcon) => {
                                this.do_send(rcon::RconUp {
                                    rcon
                                });
                            },
                            Err(e) => {
                                log::error!("cannot connect to rcon: {}", e);
                                this.do_send(instance_messages::Kill);
                            }
                        }
                    });
                };
                Ok(())
            },
            bs => {
                log::error!("cannot switch server in bad state");
                self.state = bs;
                return Err(anyhow!("cannot switch server in bad state"));
            }
        }
    }
}

impl Handler<rcon::RconUp> for Instance {
    type Result = anyhow::Result<()>;

    fn handle(&mut self, msg: rcon::RconUp, _: &mut Self::Context) -> Self::Result {
        match std::mem::replace(&mut self.state, InstanceState::Swap) {
            InstanceState::Starting { data, child } => {
                self.state = InstanceState::Running {
                    child,
                    rcon: msg.rcon,
                    data
                };
                Ok(())
            },
            os => {
                self.state = os;
                log::error!("rcon is not available for {:?}", &self.place);
                Err(anyhow!("rcon is not available for {:?}", &self.place))
            }
        }
    }
}

impl Handler<rcon::RconDown> for Instance {
    type Result = ();

    fn handle(&mut self, _: rcon::RconDown, _: &mut Self::Context) -> Self::Result {
        match std::mem::replace(&mut self.state, InstanceState::Swap) {
            InstanceState::Running { mut child, data, .. } | InstanceState::Starting { mut child, data } => {
                let _ = child.kill();
                utils::dispose(child);
                self.state = InstanceState::Crashed { data };
            },
            os => {
                self.state = os;
                log::error!("rcon is not available for {:?}", &self.place);
            }
        }
    }
}

impl Handler<rcon::RconSubscription> for Instance {
    type Result = anyhow::Result<rcon::RconStream>;

    fn handle(&mut self, _msg: rcon::RconSubscription, _ctx: &mut Self::Context) -> Self::Result {
        match &mut self.state {
            InstanceState::Running { rcon, .. } => {
                Ok(Box::pin(rcon.output_stream()))
            },
            _ => {
                log::error!("rcon is not available for {:?}", &self.place);
                Err(anyhow!("rcon is not available for {:?}", &self.place))
            }
        }
    }
}

impl Handler<instance_messages::AlterServer> for Instance {
    type Result = anyhow::Result<()>;

    fn handle(&mut self, msg: instance_messages::AlterServer, _: &mut Self::Context) -> Self::Result {

        let mfest = match &mut self.state {
            InstanceState::Crashed { data } => data,
            InstanceState::Stopped { data } => data,
            _ => {
                log::error!("cannot alter server in bad state");
                return Err(anyhow!("cannot alter server in bad state"));
            }
        };

        if let Some(port) = msg.port {
            mfest.desc.ports.port = port;
        }

        if let Some(max_memory) = msg.max_memory {
            mfest.desc.max_memory = max_memory;
        }

        if let Some(java_args) = msg.java_args {
            mfest.desc.java_args = java_args;
        }

        mfest.desc.flush(&mut mfest.manifest)?;

        Ok(())
    }
}
