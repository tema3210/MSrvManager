use std::{fs::File, path::PathBuf};

use serde::{Deserialize,Serialize};

use async_graphql::Enum;

#[derive(Copy, Clone, PartialEq, Eq, Enum,Deserialize, Serialize,Debug)]
pub enum ServerState {
    Running,
    Stopped,
    Crashed
}

#[derive(Clone, Deserialize, Serialize,Debug)]
pub struct InstanceDescriptor {
    pub name: String,
    pub mods: url::Url,
    pub state: ServerState,
    // in GB
    pub memory: Option<f64>,
    // in GB
    pub max_memory: f64,
    pub port: u16,

    pub server_jar: PathBuf,
    pub rcon: u16,
}

impl InstanceDescriptor {
    pub fn to_file(&self,file: &mut File) -> anyhow::Result<()> {
        Ok(serde_json::to_writer(file, self)?)
    }

    pub fn from_file(file: &mut File) -> anyhow::Result<Self> {
        Ok(serde_json::from_reader(file)?)
    }
}