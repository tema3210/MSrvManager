use anyhow::anyhow;

use async_graphql::{EmptySubscription, InputObject, Schema };

use async_graphql::{Context, Object};

use crate::*;

pub struct Query;

#[Object]
impl Query {
    async fn api_version(&self) -> &'static str {
        "0.1"
    }

    async fn servers<'cx>(&self,ctx: &Context<'cx>) -> Result<Vec<model::InstanceDesc>,anyhow::Error> {
        let service = ctx.data_unchecked::<native::Service>();

        Ok(service.send(messages::Instances).await?)
    }
}

#[derive(InputObject)]
pub struct InstanceCommands {
    /// Must not spawn detached processes
    pub up: String,
    pub setup: String
}

pub struct Mutation;

#[Object]
impl Mutation {

    async fn new_server<'cx>(&self,ctx: &Context<'cx>,name: String, cmds: InstanceCommands,url: url::Url, max_memory: f64, port: u16) -> Result<bool,anyhow::Error> {
        let service = ctx.data_unchecked::<native::Service>();
        
        Ok(service.send(messages::NewServer {
            name,
            up_cmd: cmds.up,
            setup_cmd: cmds.setup,
            url,
            max_memory,
            port
        }).await?)
    }

    async fn alter_server<'cx>(
        &self,
        ctx: &Context<'cx>,
        mut name: String,
        new_name: Option<String>, 
        max_memory: Option<f64>, 
        run: Option<bool>
    ) -> Result<String,anyhow::Error> {
        let service = ctx.data_unchecked::<native::Service>();

        if let Some(memory) = max_memory {
            service.send(messages::AlterServer {
                name: name.clone(),
                change: messages::ServerChange::MaxMemory(memory)
            }).await?;
        }
        if let Some(should_run) = run {
            let state = if should_run {
                model::ServerState::Running
            } else {
                model::ServerState::Stopped
            };
            service.send(messages::AlterServer {
                name: name.clone(),
                change: messages::ServerChange::Run(state)
            }).await?;
        }

        if let Some(new_name) = new_name {
            service.send(messages::AlterServer {
                name: name.clone(),
                change: messages::ServerChange::NewName(new_name.clone())
            }).await?;
            name = new_name;
        }

        Ok(name)
    }

    async fn delete_server<'cx>(&self,ctx: &Context<'cx>,name: String) -> Result<bool,anyhow::Error> {
        let service = ctx.data_unchecked::<native::Service>();

        Ok(service.send(messages::DeleteServer {
            name
        }).await?)
    }
}


// A root schema consists of a query and a mutation.
// Request queries can be executed against a RootNode.
pub type SrvsSchema = Schema<Query, Mutation, EmptySubscription>;

pub fn schema(addr: crate::native::Service) -> SrvsSchema {
    Schema::build(Query,Mutation, EmptySubscription)
    .data::<native::Service>(addr)
    .finish()
}