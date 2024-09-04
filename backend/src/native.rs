use actix::prelude::*;

impl Actor for Servers {
    type Context = actix::Context<Self>;
}

pub type Service = actix::Addr<Servers>;

pub struct Servers {
    count: usize,
}

impl Servers {
    pub fn init() -> Self {
        unimplemented!()
    }
}