use async_graphql::{InputObject, Schema, Subscription, Upload };

use async_graphql::{Context, Object};
use futures::StreamExt;
use tokio::time::MissedTickBehavior;

use crate::*;

pub struct Query;

#[Object]
impl Query {
    async fn api_version(&self) -> &'static str {
        "0.6"
    }
}

#[derive(InputObject)]
pub struct InstanceCommands {
    /// Must not spawn detached processes
    pub up: String,
    pub setup: Option<String>
}

pub struct Mutation;


#[derive(async_graphql::InputObject)]
pub struct NewServer {
    name: String,
    up_cmd: String,
    setup_cmd: Option<String>,
    url: url::Url,
    max_memory: f64,
    port: u16,
    rcon: u16,
    instance_upload: Upload
}

#[Object]
impl Mutation {

    async fn new_server<'cx>(
        &self,
        ctx: &Context<'cx>,
        data: NewServer
    ) -> Result<bool,anyhow::Error> {
        let service = ctx.data_unchecked::<native::Service>();

        let val = data.instance_upload.value(ctx)?;
        
        service.send(messages::NewServer {
            name: data.name,
            up_cmd: data.up_cmd,
            setup_cmd: data.setup_cmd,
            url: data.url,
            max_memory: data.max_memory,
            port: data.port,
            rcon: data.rcon,
            instance_upload: val
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

pub struct Subscription;

#[Subscription]
impl Subscription {
    async fn servers<'cx>(&self,ctx: &Context<'cx>) -> impl futures::Stream<Item=Vec<model::InstanceDescriptor> > + 'cx {
        log::trace!("Initializing servers subscription");
        let service = ctx.data_unchecked::<native::Service>();

        tokio_stream::wrappers::IntervalStream::new({
            let mut i = tokio::time::interval(Duration::from_secs(4));
            i.set_missed_tick_behavior(MissedTickBehavior::Skip);
            i
        })
        .then(|_| async {
            //we send heartbeat - can be put out of sync
            service.do_send(messages::Tick);
            //then we ask for the data
            match service.send(messages::Instances).await {
                Ok(data) => data,
                Err(e) => {
                    log::error!("cannot get instance list: {}",e);
                    vec![]
                }
            }
        })
    }
}


// A root schema consists of a query and a mutation.
// Request queries can be executed against a RootNode.
pub type SrvsSchema = Schema<Query, Mutation, Subscription>;

pub fn schema(addr: crate::native::Service) -> SrvsSchema {
    Schema::build(Query,Mutation, Subscription)
    .data::<native::Service>(addr)
    .finish()
}