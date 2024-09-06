use std::path::Path;

use async_graphql::{Enum, SimpleObject};

#[derive(Copy, Clone, PartialEq, Eq, Enum)]
pub enum ServerState {
    Running,
    Stopped,
    Crashed
}

#[derive(SimpleObject,Clone)]
pub struct InstanceDesc {
    pub name: String,
    pub mods: url::Url,
    pub state: ServerState,
    pub memory: f64,
    pub max_memory: f64,
    pub port: u16
}

impl InstanceDesc {
    pub fn at<P: AsRef<Path>>(at: P) -> Option<Self> {
        todo!()
    }

    pub fn start<P: AsRef<Path>>(&mut self, rcon: u16, root_dir: P)  {
        todo!()
    }

    pub fn stop<P: AsRef<Path>>(&mut self,root_dir: P ) {
        todo!()
    }
}