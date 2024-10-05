use std::{ffi::OsString, fs::File, io::{Seek, SeekFrom}, path::PathBuf};

use async_graphql::SimpleObject;
use serde::{Deserialize,Serialize};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct InstanceDescriptor {
    pub name: String,
    pub mods: url::Url,
    // in GB
    pub memory: Option<f64>,
    // in GB
    pub max_memory: f64,
    pub port: u16,

    pub server_jar: PathBuf,
    pub java_args: Vec<OsString>,
    pub rcon: u16,
}

impl InstanceDescriptor {
    pub fn flush(&self,file: &mut File) -> anyhow::Result<()> {
        file.seek(SeekFrom::Start(0))?;
        file.set_len(0)?;
        Ok(serde_json::to_writer(file, self)?)
    }

    pub fn from_file(file: &mut File) -> anyhow::Result<Self> {
        Ok(serde_json::from_reader(file)?)
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Ports {
    pub port: u16,
    pub rcon: u16
}

#[derive(SimpleObject)]
pub struct PortsInfo {
    pub ports: Vec<u16>,
    pub rcons: Vec<u16>,
    pub port_limits: [u16;2],
    pub rcon_limits: [u16;2]
}

#[derive(Serialize, Debug)]
pub enum InstanceState {
    // can be acted upon
    Running,
    Stopped,
    Crashed,

    // not displayed in UI
    Starting,
    Downloading,
    Busy
}