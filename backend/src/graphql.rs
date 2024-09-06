use anyhow::anyhow;

use async_graphql::{EmptySubscription, InputObject, Schema };

use async_graphql::{Context, Object};

use crate::native::Service;

use crate::messages;

use crate::model::*;
pub struct Query;

#[Object]
impl Query {
    async fn api_version(&self) -> &'static str {
        "0.1"
    }

    async fn servers<'cx>(&self,ctx: &Context<'cx>) -> Result<Vec<InstanceDesc>,anyhow::Error> {
        let service = ctx.data_unchecked::<Service>();

        Ok(service.send(messages::Instances).await?)
    }
}

#[derive(InputObject)]
pub struct InstanceCommands {
    /// Must not spawn detached processes
    pub up: String,
}

pub struct Mutation;

#[Object]
impl Mutation {

    async fn new_server<'cx>(&self,ctx: &Context<'cx>,name: String, cmds: InstanceCommands,url: url::Url, max_memory: f64) -> Result<bool,anyhow::Error> {
        let cx = ctx.data_unchecked::<Service>();

        Err(anyhow!("not yet implemented"))
    }

    async fn alter_server<'cx>(
        &self,
        ctx: &Context<'cx>,
        name: String, 
        new_name: Option<String>, 
        max_memory: Option<f64>, 
        run: Option<bool>
    ) -> Result<String,anyhow::Error> {
        let cx = ctx.data_unchecked::<Service>();

        Err(anyhow!("not yet implemented"))
    }

    async fn delete_server<'cx>(&self,ctx: &Context<'cx>) -> Result<bool,anyhow::Error> {
        let cx = ctx.data_unchecked::<Service>();

        Err(anyhow!("not yet implemented"))
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