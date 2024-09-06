use std::path::PathBuf;

use crate::messages;

use actix::prelude::*;

impl Actor for Servers {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("Native service is alive");

        // here we start watcher thread
        let address = ctx.address();
    }
}

pub type Service = actix::Addr<Servers>;

pub struct Servers {
    serversDir: PathBuf,
}

impl Servers {
    pub fn init<P: Into<PathBuf>>(path: P) -> Self {
        Self { serversDir: path.into() }
    }
}

impl Handler<messages::Instances> for Servers {
    type Result = MessageResult<messages::Instances>;

    fn handle(&mut self, _: messages::Instances, ctx: &mut Context<Self>) -> Self::Result {
        println!("Ping received");

        MessageResult(vec![
            crate::model::InstanceDesc {
                name: "Dummy".into(),
                state: crate::model::ServerState::Stopped,
                memory: 0.0,
                max_memory: 6.0
            }
        ])
    }
}
