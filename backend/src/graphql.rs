use async_graphql::{EmptySubscription, InputObject, Schema};

use async_graphql::{Context, Object};

use crate::native::Service;

use crate::model::*;

pub struct Query;

#[Object]
impl Query {
    async fn api_version(&self) -> &'static str {
        "0.1"
    }

    async fn servers<'cx>(&self,ctx: &Context<'cx>) -> Option<InstanceDesc> {
        let ctx = ctx.data_unchecked::<Service>();
        Some(InstanceDesc {
            name: "Dummy".into(),
            state: ServerState::Stopped,
            memory: 0.0,
            max_memory: 6.0
        })
    }
}


#[derive(InputObject)]
pub struct CreateServer {
    pub name: String,
    pub modpack: String,
}


pub struct Mutation;

#[Object]
impl Mutation {

    async fn new_server<'cx>(&self,ctx: &Context<'cx>,req: CreateServer) -> Option<String> {
        let cx = ctx.data_unchecked::<Service>();
        unimplemented!()
    }
}


// A root schema consists of a query and a mutation.
// Request queries can be executed against a RootNode.
pub type SrvsSchema = Schema<Query, Mutation, EmptySubscription>;

pub fn schema(addr: crate::native::Service) -> SrvsSchema {
    Schema::build(Query,Mutation, EmptySubscription)
    .data::<Service>(addr)
    .finish()
}