use async_graphql::{Enum, SimpleObject};


#[derive(Copy, Clone, PartialEq, Eq, Enum)]
pub enum ServerState {
    Running,
    Stopped,
    Crashed
}

#[derive(SimpleObject)]
pub struct InstanceDesc {
    pub name: String,
    pub state: ServerState,
    pub memory: f64,
    pub max_memory: f64
}