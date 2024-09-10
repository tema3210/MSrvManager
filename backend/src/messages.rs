use std::path::Path;

use actix::Message;

use crate::*;

/// Define message
#[derive(Message)]
#[rtype(result = "Vec<model::InstanceDescriptor>")]
pub struct Instances;

#[derive(Message)]
#[rtype(result = "anyhow::Result<()>")]
pub struct NewServer {
    pub name: String,
    pub up_cmd: String,
    pub setup_cmd: Option<String>,
    pub url: url::Url,
    pub max_memory: f64,
    pub port: u16,
    pub rcon: u16
}

#[derive(Message)]
#[rtype(result = "anyhow::Result<()>")]
pub struct DeleteServer {
    pub name: String
}


#[derive(Message)]
#[rtype(result = "anyhow::Result<()>")]
pub struct AlterServer {
    pub name: String,
    pub change: model::ServerChange
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Tick;

#[derive(Message)]
#[rtype(result = "()")]
pub struct LoadingEnded(pub Arc<Path>,pub Option<anyhow::Error>);