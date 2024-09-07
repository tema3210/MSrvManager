use std::{collections::HashMap, ops::Range, path::PathBuf};

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
}

const PORT_RANGE: Range<u16> = 25000..63000;

pub struct Servers {
    servers_dir: PathBuf,
    rcon_range: Indices,
    port_range: Indices,
    servers: HashMap<PathBuf,model::InstanceDesc>
}

impl Servers {
    pub fn path_to_name(&self, name: String) -> PathBuf {
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
                        if let Some(instance) = model::InstanceDesc::at(&e_path) {

                            if let Err(e) = port_range.try_take(instance.port) {
                                log::error!("a servers {} port {}",&instance.name,e);
                                return None;
                            }

                            if let Err(e) = rcon_range.try_take(instance.rcon) {
                                log::error!("a servers {} rcon port {}",&instance.name,e);
                                return None;
                            }

                            Some((e_path,instance))
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
            log::info!("hb of {}",&i.name);
        }
    }
}

impl Actor for Servers {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // here we start watcher thread
        let address = ctx.address();

        for (_,i) in self.servers.iter_mut() {
            i.start(&self.servers_dir);
        }
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        for (_,i) in &mut self.servers {
            i.stop(&self.servers_dir)
        }
        Running::Stop
    }
}

pub type Service = actix::Addr<Servers>;

impl Handler<messages::Instances> for Servers {
    type Result = MessageResult<messages::Instances>;

    fn handle(&mut self, _: messages::Instances, _: &mut Context<Self>) -> Self::Result {
        self.hb();
        MessageResult(self.servers.values().cloned().collect())
    }
}

impl Handler<messages::NewServer> for Servers {
    type Result = MessageResult<messages::NewServer>;

    fn handle(&mut self, msg: messages::NewServer, ctx: &mut Self::Context) -> Self::Result {
        todo!()
    }
}

impl Handler<messages::AlterServer> for Servers {
    type Result = MessageResult<messages::AlterServer>;

    fn handle(&mut self, msg: messages::AlterServer, ctx: &mut Self::Context) -> Self::Result {
        todo!()
    }
}

impl Handler<messages::DeleteServer> for Servers {
    type Result = MessageResult<messages::DeleteServer>;

    fn handle(&mut self, msg: messages::DeleteServer, ctx: &mut Self::Context) -> Self::Result {
        todo!()
    }
}