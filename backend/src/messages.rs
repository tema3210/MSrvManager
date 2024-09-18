use std::path::Path;

use actix::Message;
use async_graphql::UploadValue;

use crate::*;

/// Define message
#[derive(Message,Debug)]
#[rtype(result = "Vec<model::InstanceDescriptor>")]
pub struct Instances;

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