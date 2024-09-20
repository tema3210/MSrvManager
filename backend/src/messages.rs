use std::path::Path;

use actix::Message;
use async_graphql::{SimpleObject, UploadValue};

use crate::*;


#[derive(Message,Debug)]
#[rtype(result = "Vec<O>")]
pub struct Instances<O: Send + 'static,F: Send + Fn(&model::InstanceDescriptor) -> O>{
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
    pub change: model::ServerChange
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