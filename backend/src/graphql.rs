use async_graphql::{EmptySubscription, InputObject, Schema };

use async_graphql::{Context, Object};

use crate::*;

pub struct Query;

#[Object]
impl Query {
    async fn api_version(&self) -> &'static str {
        "0.5"
    }

    async fn servers<'cx>(&self,ctx: &Context<'cx>) -> Result<Vec<model::InstanceDescriptor>,anyhow::Error> {
        let service = ctx.data_unchecked::<native::Service>();

        Ok(service.send(messages::Instances).await?)
    }
}

#[derive(InputObject)]
pub struct InstanceCommands {
    /// Must not spawn detached processes
    pub up: String,
    pub setup: Option<String>
}

pub struct Mutation;

#[Object]
impl Mutation {

    async fn new_server<'cx>(&self,ctx: &Context<'cx>,name: String, cmds: InstanceCommands,url: url::Url, max_memory: f64, port: u16, rcon: u16) -> Result<bool,anyhow::Error> {
        let service = ctx.data_unchecked::<native::Service>().clone();
        
        service.send(messages::NewServer {
            name,
            up_cmd: cmds.up,
            setup_cmd: cmds.setup,
            url,
            max_memory,
            port,
            rcon
        }).await??;

        Ok(true)

    }

    async fn alter_server<'cx>(
        &self,
        ctx: &Context<'cx>,
        name: String,
        max_memory: Option<f64>,
        run: Option<bool>,
        port: Option<u16>,
        rcon: Option<u16>
    ) -> Result<String,anyhow::Error> {
        let service = ctx.data_unchecked::<native::Service>();

        if let Some(memory) = max_memory {
            service.send(messages::AlterServer {
                name: name.clone(),
                change: model::ServerChange::MaxMemory(memory)
            }).await??;
        }
        if let Some(should_run) = run {
            service.send(messages::AlterServer {
                name: name.clone(),
                change: model::ServerChange::Run(should_run)
            }).await??;
        }

        if let Some(port) = port {
            service.send(messages::AlterServer {
                name: name.clone(),
                change: model::ServerChange::Port(port)
            }).await??;
        }

        if let Some(rcon) = rcon {
            service.send(messages::AlterServer {
                name: name.clone(),
                change: model::ServerChange::Rcon(rcon)
            }).await??;
        }

        Ok(name)
    }

    async fn delete_server<'cx>(&self,ctx: &Context<'cx>,name: String) -> Result<bool,anyhow::Error> {
        let service = ctx.data_unchecked::<native::Service>();

        service.send(messages::DeleteServer {
            name
        }).await??;
        Ok(true)
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