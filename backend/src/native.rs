use std::{ops::Range, path::PathBuf};

use crate::*;

use actix::prelude::*;

#[derive(Debug,Clone)]
pub struct RconRange(pub Range<u16>);

impl RconRange {
    fn next(&mut self) -> Option<u16> {
        if self.0.is_empty() {
            return None
        };
        let start = self.0.start;
        self.0.start += 1;
        Some(start)
    }
}

pub enum Servers {
    Up {
        servers_dir: PathBuf,
        rcon_range: RconRange,
        servers: Vec<model::InstanceDesc>
    },
    Stopped {

    }
}

impl Servers {
    pub fn init<P: Into<PathBuf>>(path: P, rcon_range: Range<u16>) -> Option<Self> {
        let servers_dir = path.into();

        let Ok(servers) = std::fs::read_dir(&servers_dir) else {
            return None
        };

        let servers: Vec<_> = servers.filter_map(|e| {
            match e {
                Ok(e) => {
                    let e_path = e.path();
                    if e_path.is_dir() {
                        model::InstanceDesc::at(e_path)
                    } else {
                        None
                    }
                },
                Err(_) => None
            }
        }).collect();

        Some(Self::Up { servers_dir, rcon_range: RconRange(rcon_range), servers })
    }
}

impl Actor for Servers {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {

        // here we start watcher thread
        let address = ctx.address();

        match self {
            Servers::Up { servers_dir, rcon_range, servers } => {
                for i in servers.iter_mut() {
                    if let Some(rcon) = rcon_range.next() {
                        i.start(rcon,&servers_dir);
                    }
                }
            },
            Servers::Stopped {  } => panic!("Cannot start after we've stopped")
        }

    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        match self {
            Servers::Up { servers, servers_dir, .. } => {
                for i in servers {
                    i.stop(&servers_dir)
                }
                *self = Servers::Stopped {}
            },
            _ => {}
        }
        Running::Stop
    }
}

pub type Service = actix::Addr<Servers>;

impl Handler<messages::Instances> for Servers {
    type Result = MessageResult<messages::Instances>;

    fn handle(&mut self, _: messages::Instances, _: &mut Context<Self>) -> Self::Result {

        match self {
            Servers::New { .. } => MessageResult(vec![]),
            Servers::Up { servers, .. } => MessageResult(servers.clone()),
            Servers::Stopped { .. } => MessageResult(vec![]),
        }
    }
}

impl Handler<messages::NewServer> for Servers {
    type Result = MessageResult<messages::NewServer>;

    fn handle(&mut self, msg: messages::NewServer, ctx: &mut Self::Context) -> Self::Result {
        match self {
            Servers::Up { servers_dir, rcon_range, servers } => {
                todo!()
            },
            Servers::Stopped {  } => MessageResult(false),
        }
    }
}

impl Handler<messages::AlterServer> for Servers {
    type Result = MessageResult<messages::AlterServer>;

    fn handle(&mut self, msg: messages::AlterServer, ctx: &mut Self::Context) -> Self::Result {
        match self {
            Servers::Up { servers_dir, rcon_range, servers } => {
                todo!()
            },
            Servers::Stopped {  } => MessageResult(false),
        }
    }
}

impl Handler<messages::DeleteServer> for Servers {
    type Result = MessageResult<messages::DeleteServer>;

    fn handle(&mut self, msg: messages::DeleteServer, ctx: &mut Self::Context) -> Self::Result {
        match self {
            Servers::Up { servers_dir, rcon_range, servers } => {
                todo!()
            },
            Servers::Stopped {  } => MessageResult(false),
        }
    }
}