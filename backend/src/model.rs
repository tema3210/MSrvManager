use std::{fs::File, io::{Seek, SeekFrom}};

use async_graphql::SimpleObject;
use serde::{Deserialize,Serialize};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct InstanceDescriptor {
    pub name: String,

    pub mods: url::Url,
    pub java_args: Vec<String>,
    // pub server_jar: PathBuf,

    // in GB
    pub memory: Option<f64>,
    // in GB
    pub max_memory: f64,

    pub ports: Ports,
}

#[derive(Debug)]
pub enum IDError {
    JSON(serde_json::Value),
    IO(serde_json::Error)
}

impl InstanceDescriptor {
    pub fn flush(&self,file: &mut File) -> anyhow::Result<()> {
        file.seek(SeekFrom::Start(0))?;
        file.set_len(0)?;
        Ok(serde_json::to_writer(file, self)?)
    }

    pub fn from_file(file: &mut File) -> Result<Self,IDError> {
        if let Ok(id) = serde_json::from_reader::<_,Self>(&mut *file) {
            return Ok(id)
        };
        let val: serde_json::Value = serde_json::from_reader(file)
            .map_err(|e| IDError::IO(e))?;
        Err(IDError::JSON(val))
    }
}

#[derive(Clone,Copy, Deserialize, Serialize, Debug, async_graphql::InputObject)]
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