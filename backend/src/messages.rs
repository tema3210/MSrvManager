use actix::Message;

use crate::*;

/// Define message
#[derive(Message)]
#[rtype(result = "Vec<model::InstanceDesc>")]
pub struct Instances;

#[derive(Message)]
#[rtype(result = "bool")]
pub struct NewServer {
    pub name: String,
    pub up_cmd: String,
    pub setup_cmd: String,
    pub url: url::Url,
    pub max_memory: f64,
    pub port: u16,
    pub rcon: u16
}

#[derive(Message)]
#[rtype(result = "bool")]
pub struct DeleteServer {
    pub name: String
}


#[derive(Message)]
#[rtype(result = "bool")]
pub struct AlterServer {
    pub name: String,
    pub change: model::ServerChange
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Tick;