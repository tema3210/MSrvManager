use std::path::Path;

use actix::Message;
use async_graphql::{SimpleObject, UploadValue};

use crate::*;


#[derive(Message,Debug)]
#[rtype(result = "Vec<O>")]
pub struct Instances<O: Send + 'static,F: Send + Fn(&model::InstanceDescriptor) -> O>{
    pub f: F
}

#[derive(Message,Debug)]
#[rtype(result = "Option<O>")]
pub struct Instance<O: Send + 'static,F: Send + Fn(&model::InstanceDescriptor) -> O>{
    pub name: String,
    pub f: F
}

#[derive(SimpleObject)]
pub struct PortsInfo {
    pub ports: Vec<u16>,
    pub rcons: Vec<u16>,
    pub port_limits: [u16;2],
    pub rcon_limits: [u16;2]
}

#[derive(Message,Debug)]
#[rtype(result = "PortsInfo")]
pub struct Ports;

#[derive(Message)]
#[rtype(result = "anyhow::Result<()>")]
pub struct NewServer {
    pub name: String,
    pub up_cmd: String,
    pub setup_cmd: Option<String>,
    pub url: url::Url,
    pub instance_upload: UploadValue,
    pub max_memory: f64,
    pub port: u16,
    pub rcon: u16
}

impl std::fmt::Debug for NewServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f
            .debug_struct("NewServer")
            .field("name", &self.name)
            .field("up_cmd", &self.up_cmd)
            .field("setup_cmd", &self.setup_cmd)
            .field("url", &self.url)
            .field("instance_upload", &"UploadValue")
            .field("max_memory", &self.max_memory)
            .field("port", &self.port)
            .field("rcon", &self.rcon)
            .finish()
    }
}

#[derive(Message,Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct DeleteServer {
    pub name: String
}

#[derive(Message,Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct SwitchServer {
    pub name: String,
    pub should_run: bool
}

#[derive(Message,Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct AlterServer {
    pub name: String,
    pub max_memory: Option<f64>,
    pub port: Option<u16>,
    pub up_cmd: Option<String>
}

#[derive(Message,Debug)]
#[rtype(result = "()")]
pub struct Tick;

#[derive(Message,Debug)]
#[rtype(result = "()")]
pub struct LoadingEnded(pub Arc<Path>,pub Option<anyhow::Error>);

#[derive(Message,Debug)]
#[rtype(result = "()")]
pub struct InstanceStopped(pub Arc<Path>);