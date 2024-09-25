use std::{
    collections::HashMap,
    fs::{File, Permissions},
    io::{copy, Write},
    ops::Range,
    path::{Path, PathBuf},
};
use zip::ZipArchive;

use anyhow::anyhow;
use instance::Instance;

use crate::*;

use actix::prelude::*;

#[derive(Debug)]
pub struct Indices(Range<u16>, bit_set::BitSet);

impl Indices {
    fn try_take(&mut self, idx: u16) -> Result<(), anyhow::Error> {
        if !self.0.contains(&idx) {
            return Err(anyhow!("out of bounds"));
        };

        if self.1.contains(idx.into()) {
            return Err(anyhow!("already occupied"));
        };

        self.1.insert(idx.into());

        Ok(())
    }

    fn free(&mut self, idx: u16) -> anyhow::Result<()> {
        if !self.0.contains(&idx) {
            return Err(anyhow!("out of bounds"));
        };

        if self.1.contains(idx.into()) {
            self.1.remove(idx.into());
            return Ok(());
        };

        Err(anyhow!("already freed"))
    }

    /// iterate over taken ports
    pub fn taken(&self) -> Vec<u16> {
        self.1
            .iter()
            .map(|p| p.try_into().expect("have port larger than it should be"))
            .collect()
    }
}

pub struct Servers {
    servers_dir: PathBuf,
    rcon_range: Indices,
    port_range: Indices,
    servers: HashMap<std::sync::Arc<Path>, instance::Instance>,
}

impl Servers {
    pub fn name_to_path<P: AsRef<Path>>(&self, name: P) -> PathBuf {
        let mut res = self.servers_dir.clone();
        res.push(name.as_ref());
        res
    }

    pub fn init<P: Into<PathBuf>>(
        path: P,
        rcon_range: Range<u16>,
        port_range: Range<u16>,
    ) -> Option<Self> {
        let servers_dir = path.into();

        let Ok(servers) = std::fs::read_dir(&servers_dir) else {
            return None;
        };

        let mut rcon_range = Indices(
            rcon_range.clone(),
            bit_set::BitSet::with_capacity(rcon_range.len()),
        );
        let mut port_range = Indices(port_range, bit_set::BitSet::with_capacity(u16::MAX.into()));

        let servers: HashMap<_, _> = servers
            .filter_map(|e| match e {
                Ok(e) => {
                    let e_path = e.path();
                    if e_path.is_dir() {
                        let arc_path: Arc<Path> = e_path.into();
                        if let Ok(instance) = instance::Instance::load(Arc::clone(&arc_path)) {
                            let desc = &instance.desc;

                            if let Err(e) = port_range.try_take(desc.port) {
                                log::error!("a servers {} port {}", &desc.name, e);
                                return None;
                            }

                            if let Err(e) = rcon_range.try_take(desc.rcon) {
                                log::error!("a servers {} rcon port {}", &desc.name, e);
                                return None;
                            }

                            Some((arc_path.clone(), instance))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                Err(_) => None,
            })
            .collect();

        Some(Self {
            servers_dir,
            rcon_range,
            port_range,
            servers,
        })
    }

    pub fn hb(&mut self) {
        for (_, i) in &mut self.servers {
            i.hb();
        }
    }

    pub fn nuke(&mut self, instance: &instance::Instance) -> anyhow::Result<()> {
        self.port_range.1.remove(instance.desc.port.into());
        self.rcon_range.1.remove(instance.desc.rcon.into());
        std::fs::remove_dir_all(instance.place.as_ref())?;
        Ok(())
    }
}

impl Actor for Servers {
    type Context = actix::Context<Self>;

    fn started(&mut self, _: &mut Self::Context) {
        for (_, i) in &mut self.servers {
            match i.desc.state {
                model::ServerState::Running => i.start(),
                _ => {}

            }
        }
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        for (_, i) in &mut self.servers {
            i.stop();
        }
        Running::Stop
    }
}

pub type Service = actix::Addr<Servers>;

impl Handler<messages::Ports> for Servers {
    type Result = MessageResult<messages::Ports>;

    fn handle(&mut self, _: messages::Ports, _: &mut Self::Context) -> Self::Result {
        let pr = &self.port_range.0;
        let rr = &self.rcon_range.0;
        MessageResult(messages::PortsInfo {
            ports: self.port_range.taken(),
            rcons: self.rcon_range.taken(),
            port_limits: [pr.start, pr.end],
            rcon_limits: [rr.start, rr.end],
        })
    }
}

impl<O, F> Handler<messages::Instances<O, F>> for Servers
where
    F: Send + Fn(&model::InstanceDescriptor) -> O,
    O: Send + 'static,
{
    type Result = Vec<O>;

    fn handle(&mut self, m: messages::Instances<O, F>, _: &mut Context<Self>) -> Self::Result {
        self
            .servers
            .values()
            .filter(|i| matches!(i.instance_state, instance::InstanceState::Normal))
            .map(|i| &i.desc)
            .map(m.f)
            .collect()
    }
}

impl Handler<messages::LoadingEnded> for Servers {
    type Result = ();

    fn handle(&mut self, msg: messages::LoadingEnded, _: &mut Self::Context) -> Self::Result {
        match self.servers.entry(msg.0) {
            std::collections::hash_map::Entry::Occupied(mut e) => {
                if !matches!(e.get().instance_state, instance::InstanceState::Downloading) {
                    log::error!(
                        "loading ended recieved for a non loading instance at {:?}",
                        e.key()
                    );
                    return;
                }
                if let Some(error) = msg.1 {
                    let instance = e.remove();
                    let name = &instance.desc.name;
                    log::error!("couldn't load {name} due to {error}");
                    if let Err(e) = self.nuke(&instance) {
                        log::error!("cannot nuke {name}: {e}")
                    }
                    return;
                };
                e.get_mut().instance_state = instance::InstanceState::Normal;
            }
            std::collections::hash_map::Entry::Vacant(ve) => {
                let name = &**ve.key();
                log::error!("loading event on unexisting server called {name:?}");
            }
        }
    }
}

impl Handler<messages::NewServer> for Servers {
    type Result = anyhow::Result<()>;

    fn handle(&mut self, msg: messages::NewServer, ctx: &mut Self::Context) -> Self::Result {
        let path = self.name_to_path(&msg.name);

        if path.exists() && self.servers.contains_key((&*path).into()) {
            return Err(anyhow!("server name is already in use"));
        }

        log::info!("creating server with {:?}", &msg);

        match (
            self.rcon_range.try_take(msg.rcon),
            self.port_range.try_take(msg.port),
        ) {
            //rcon   port
            (Ok(()), Ok(())) => {}
            (Ok(()), Err(e)) => {
                self.rcon_range.free(msg.rcon).unwrap(); // free rcon back
                return Err(e);
            }
            (Err(e), Ok(())) => {
                self.port_range.free(msg.port).unwrap(); // free port back
                return Err(e);
            }
            (Err(_), Err(_)) => return Err(anyhow!("both rcon and port are bad")),
        };

        log::info!("create server at {:?}", &*path);

        let desc: model::InstanceDescriptor = model::InstanceDescriptor {
            name: msg.name,
            mods: msg.url,
            state: model::ServerState::Stopped,
            max_memory: msg.max_memory,
            memory: None,
            port: msg.port,
            rcon: msg.rcon,
        };

        //make an instance

        Instance::prepare(&path)?;

        let instance_place: Arc<Path> = path.into();

        //make a command file
        {
            let mut cmd_file = File::options()
                .write(true)
                .open(&*instance_place.join(instance::COMMAND_FILE_NAME))?;

            cmd_file.write(msg.up_cmd.as_bytes())?;

            drop(cmd_file);

        }

        let instance = Instance::create(
            Arc::clone(&instance_place),
            desc,
            instance::InstanceState::Downloading,
        )?;

        self.servers.insert(Arc::clone(&instance_place), instance);

        std::thread::spawn({
            let addr = ctx.address();
            let instance_place = instance_place;

            let output_dir = Arc::clone(&instance_place);

            let mut instance_data = msg.instance_upload.content;

            let setup_cmd = msg.setup_cmd.map(|c| {
                let mut cmd = std::process::Command::new(c);
                cmd.current_dir(&*instance_place);
                cmd
            });

            let name = Arc::clone(&instance_place);
            let job = move || -> anyhow::Result<()> {
                let mut archive = ZipArchive::new(&mut instance_data)?;

                log::info!("starting to unpack to {:?}", &*name);

                for i in 0..archive.len() {
                    let mut archive_file = archive.by_index(i)?;

                    let outpath = match archive_file.enclosed_name() {
                        Some(path) => (&*output_dir).join(path),
                        None => continue,
                    };

                    // Create directories if necessary
                    if archive_file.is_dir() {
                        log::trace!("creating dir {:?}", &*outpath);
                        let _ = std::fs::create_dir_all(&outpath)?;
                    } else {
                        if let Some(p) = outpath.parent() {
                            log::trace!("create dir all at {:?} for {:?}", p, &outpath);
                            let _ = std::fs::create_dir_all(p)?;
                        }
                        log::trace!("making {:?}", &*outpath);
                        let mut outfile = File::create(&outpath)?;
                        log::trace!("copying {:?}", &*outpath);

                        copy(&mut archive_file, &mut outfile)?;

                        // Set file permissions
                        if let Some(mode) = archive_file.unix_mode() {
                            let permissions = <Permissions as std::os::unix::fs::PermissionsExt>::from_mode(mode);
                            std::fs::set_permissions(&outpath, permissions.clone())?;
                            log::info!("set permissions {:?} for {:?}", permissions, &*outpath);
                        }

                        log::info!("copied {:?}", &*outpath);
                    }
                }

                //run setup command if it exists
                if let Some(mut c) = setup_cmd {
                    if c.spawn()?.wait()?.success() {
                        Ok(())
                    } else {
                        Err(anyhow!("the setup command didn't succeed"))
                    }
                } else {
                    Ok(())
                }
            };
            move || match job() {
                Ok(()) => {
                    addr.do_send(messages::LoadingEnded(instance_place, None));
                }
                Err(e) => {
                    addr.do_send(messages::LoadingEnded(instance_place, Some(e)));
                }
            }
        });

        Ok(())
    }
}

impl Handler<messages::SwitchServer> for Servers {
    type Result = anyhow::Result<()>;

    fn handle(&mut self, msg: messages::SwitchServer, ctx: &mut Self::Context) -> Self::Result {
        let path = self.name_to_path(msg.name);

        match self.servers.get_mut(&*path) {
            Some(instance) => {
                if !matches!(instance.instance_state, instance::InstanceState::Normal) {
                    log::error!("cannot switch server in bad state");
                    return Err(anyhow!("cannot switch server in bad state"));
                }
                let t = (instance.desc.state,msg.should_run);
                log::trace!("switching {:?} on {:?}", &path, t);
                match t {
                    (model::ServerState::Stopped | model::ServerState::Crashed, true) => {
                        instance.start();
                    }
                    (model::ServerState::Running, false) => {
                        instance.stop_async(ctx.address());
                    }
                    _ => {}
                }
                Ok(())
            }
            None => Err(anyhow!("cannot switch unexisting server")),
        }
    }
}

impl Handler<messages::AlterServer> for Servers {
    type Result = anyhow::Result<()>;

    fn handle(&mut self, msg: messages::AlterServer, _: &mut Self::Context) -> Self::Result {
        let path = self.name_to_path(msg.name);

        match self.servers.get_mut((&*path).into()) {
            Some(instance) => {
                match msg.change {
                    model::ServerChange::MaxMemory(mm) => instance.desc.max_memory = mm,
                    model::ServerChange::Port(port) => match self.port_range.try_take(port) {
                        Ok(_) => {
                            self.port_range.free(instance.desc.port)?;
                            instance.desc.port = port
                        }
                        Err(_) => return Err(anyhow!("cannot change port to blacklisted")),
                    },
                    model::ServerChange::Rcon(port) => {
                        match self.rcon_range.try_take(port) {
                            Ok(_) => {
                                self.rcon_range.free(instance.desc.port)?;
                                instance.desc.port = port
                            }
                            Err(_) => return Err(anyhow!("cannot change port to blacklisted")),
                        }
                        instance.desc.rcon = port;
                    }
                };
                instance.flush();
                Ok(())
            }
            None => Err(anyhow!("cannot change port to blacklisted")),
        }
    }
}

impl Handler<messages::InstanceStopped> for Servers {
    type Result = ();

    fn handle(&mut self, msg: messages::InstanceStopped, _: &mut Self::Context) -> Self::Result {
        match self.servers.entry(msg.0) {
            std::collections::hash_map::Entry::Occupied(mut e) => {
                if !matches!(e.get().instance_state, instance::InstanceState::Stopping) {
                    log::error!("instance stopped message for a not stopping target");
                    return;
                };
                log::info!("instance stopped for {:?}", e.key());
                e.get_mut().finish_stop();
            }
            std::collections::hash_map::Entry::Vacant(_) => {
                log::error!("instance stopped for unexisting");
            }
        }
    }
}

impl Handler<messages::Tick> for Servers {
    type Result = MessageResult<messages::Tick>;

    fn handle(&mut self, _: messages::Tick, _: &mut Self::Context) -> Self::Result {
        log::trace!("tick tac");
        self.hb();
        MessageResult(())
    }
}

impl Handler<messages::DeleteServer> for Servers {
    type Result = anyhow::Result<()>;

    fn handle(&mut self, msg: messages::DeleteServer, _: &mut Self::Context) -> Self::Result {
        let path = self.name_to_path(msg.name);

        match self.servers.entry(path.into()) {
            std::collections::hash_map::Entry::Occupied(e) => {
                if !matches!(e.get().instance_state, instance::InstanceState::Normal) {
                    return Err(anyhow!("Resource is in a bad state"));
                }
                let mut instance = e.remove();
                instance.kill();
                self.nuke(&instance)?;

                Ok(())
            }
            std::collections::hash_map::Entry::Vacant(_) => {
                Err(anyhow!("trying to delete absent server"))
            }
        }
    }
}
