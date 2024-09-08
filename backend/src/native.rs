use std::{collections::HashMap, ops::Range, path::{Path, PathBuf}};

use anyhow::anyhow;

use crate::*;

use actix::prelude::*;

#[derive(Debug)]
pub struct Indices(Range<u16>,bit_set::BitSet);

impl Indices {
    fn try_take(&mut self, idx: u16) -> Result<(), anyhow::Error> {
        if !self.0.contains(&idx) {
            return Err(anyhow!("out of bounds"))
        };

        if self.1.contains(idx.into()) {
            return Err(anyhow!("already occupied"))
        };

        self.1.insert(idx.into());

        Ok(())
    }

    fn free(&mut self, idx: u16) -> anyhow::Result<()> {
        if !self.0.contains(&idx) {
            return Err(anyhow!("out of bounds"))
        };

        if self.1.contains(idx.into()) {
            self.1.remove(idx.into());
            return Ok(())
        };

        Err(anyhow!("already freed"))
    }
}

const PORT_RANGE: Range<u16> = 25000..63000;

pub struct Servers {
    servers_dir: PathBuf,
    rcon_range: Indices,
    port_range: Indices,
    servers: HashMap<std::sync::Arc<Path>,instance::Instance>
}

impl Servers {
    pub fn name_to_path(&self, name: String) -> PathBuf {
        let mut res = self.servers_dir.clone();
        res.push(name);
        res
    }

    pub fn init<P: Into<PathBuf>>(path: P, rcon_range: Range<u16>) -> Option<Self> {
        let servers_dir = path.into();

        let Ok(servers) = std::fs::read_dir(&servers_dir) else {
            return None
        };

        let mut rcon_range = Indices(rcon_range.clone(),bit_set::BitSet::with_capacity(rcon_range.len()));
        let mut port_range = Indices(PORT_RANGE,bit_set::BitSet::with_capacity(u16::MAX.into()));

        let servers: HashMap<_,_> = servers.filter_map(|e| {
            match e {
                Ok(e) => {
                    let e_path = e.path();
                    if e_path.is_dir() {
                        let arc_path: Arc<Path> = e_path.into();
                        if let Some(instance) = instance::Instance::load(Arc::clone(&arc_path)) {

                            let desc = &instance.desc;

                            if let Err(e) = port_range.try_take(desc.port) {
                                log::error!("a servers {} port {}",&desc.name,e);
                                return None;
                            }

                            if let Err(e) = rcon_range.try_take(desc.rcon) {
                                log::error!("a servers {} rcon port {}",&desc.name,e);
                                return None;
                            }

                            Some((arc_path.clone(),instance))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                },
                Err(_) => None
            }
        }).collect();

        Some(Self { servers_dir, rcon_range, port_range, servers })
    }

    pub fn hb(&mut self) {
        for (p,i) in &mut self.servers {
            log::info!("hb of {} at {:?}",&i.desc.name,p);
            i.hb();
        }
    }
}

impl Actor for Servers {
    type Context = actix::Context<Self>;

    fn started(&mut self, cx: &mut Self::Context) {
        for (_,i) in &mut self.servers {
            i.start();
        };
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        for (_,i) in &mut self.servers {
            i.stop()
        }
        Running::Stop
    }
}

pub type Service = actix::Addr<Servers>;

impl Handler<messages::Instances> for Servers {
    type Result = MessageResult<messages::Instances>;

    fn handle(&mut self, _: messages::Instances, _: &mut Context<Self>) -> Self::Result {
        MessageResult(self.servers.values().map(|i| &i.desc).cloned().collect())
    }
}

impl Handler<messages::NewServer> for Servers {
    type Result = anyhow::Result<()>;

    fn handle(&mut self, msg: messages::NewServer, _: &mut Self::Context) -> Self::Result {
        let path = self.name_to_path(msg.name);
        todo!()
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
                    model::ServerChange::Port(port) => {
                        match self.port_range.try_take(port) {
                            Ok(_) => {
                                self.port_range.free(instance.desc.port)?;
                                instance.desc.port = port
                            },
                            Err(_) => return Err(anyhow!("cannot change port to blacklisted")),
                        }
                    },
                    model::ServerChange::Rcon(port) => {
                        match self.rcon_range.try_take(port) {
                            Ok(_) => {
                                self.rcon_range.free(instance.desc.port)?;
                                instance.desc.port = port
                            },
                            Err(_) => return Err(anyhow!("cannot change port to blacklisted")),
                        }
                        instance.desc.rcon = port;
                    },
                    model::ServerChange::Run(should_run) => {
                        if should_run {
                            instance.start();
                        } else {
                            instance.stop();
                        }
                        return Ok(())
                    },
                };
                instance.flush();
                Ok(())
            },
            None => Err(anyhow!("cannot change port to blacklisted")),
        }
    }
}

impl Handler<messages::Tick> for Servers {
    type Result = MessageResult<messages::Tick>;

    fn handle(&mut self, _: messages::Tick, _: &mut Self::Context) -> Self::Result {
        self.hb();
        MessageResult(())
    }
}

impl Handler<messages::DeleteServer> for Servers {
    type Result = MessageResult<messages::DeleteServer>;

    fn handle(&mut self, msg: messages::DeleteServer, _: &mut Self::Context) -> Self::Result {
        let path = self.name_to_path(msg.name);

        match self.servers.entry(path.into()) {
            std::collections::hash_map::Entry::Occupied(e) => {
                e.remove().kill();
                MessageResult(Ok(()))
            },
            std::collections::hash_map::Entry::Vacant(_) => MessageResult(Err(anyhow!("trying to delete absent server"))),
        }
    }
}