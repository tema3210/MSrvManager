use actix::Message;

use crate::model;

/// Define message
#[derive(Message)]
#[rtype(result = "Vec<model::InstanceDesc>")]
pub struct Instances;