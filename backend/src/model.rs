use serde::{Deserialize,Serialize};

use crate::*;

use async_graphql::{Enum, SimpleObject};

#[derive(Copy, Clone, PartialEq, Eq, Enum,Deserialize, Serialize)]
pub enum ServerState {
    Running,
    Stopped,
    Crashed
}

#[derive(SimpleObject,Clone, Deserialize, Serialize)]
pub struct InstanceDesc {
    pub name: String,
    pub mods: url::Url,
    pub state: ServerState,
    pub memory: f64,
    pub max_memory: f64,
    pub port: u16,
    pub rcon: u16,
}

pub enum ServerChange {
    MaxMemory(f64),
    Port(u16),
    Rcon(u16),
    Run(bool)
}