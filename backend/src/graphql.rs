use async_graphql::{Schema, Subscription, Upload };

use async_graphql::{Context, Object};
use futures::StreamExt;
use tokio::time::MissedTickBehavior;

use crate::*;

pub struct Query;

#[Object]
impl Query {
    async fn app_version(&self) -> &'static str {
        "0.9"
    }

    async fn instance<'cx>(&self, ctx: &Context<'cx>, name: String) -> anyhow::Result<Option<model::InstanceDescriptor>> {
        let service = ctx.data_unchecked::<native::Service>();

        let data =  service.send(messages::Instance { 
            name,
            f: |i| i.clone()
        }).await?;
        
        Ok(data)
    }

    async fn ports_taken<'cx>(&self, ctx: &Context<'cx>) -> anyhow::Result<messages::PortsInfo> {
        let service = ctx.data_unchecked::<native::Service>();
        Ok(service.send(messages::Ports).await?)
    }

    async fn rcons<'cx>(&self, ctx: &Context<'cx>) -> Vec<serde_json::Value> {
        let service = ctx.data_unchecked::<native::Service>();
        match service.send(messages::Instances {
            f: |i| serde_json::json!({
                "name": i.name,
                "rcon": i.rcon
            }),
        }).await {
            Ok(v) => {
                v
            },
            Err(_) => vec![]
        }

    }
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

    async fn should_run<'cx>(&self,
        ctx: &Context<'cx>,
        name: String,
        should_run: bool
    ) -> Result<bool,anyhow::Error> {
        let service = ctx.data_unchecked::<native::Service>();
        
        log::info!("should_run {} {}",name,should_run);
        service.send(messages::SwitchServer {
            name,
            should_run
        }).await??;
        Ok(true)
    }

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
        up_cmd: Option<String>,
        port: Option<u16>,
    ) -> Result<String,anyhow::Error> {
        let service = ctx.data_unchecked::<native::Service>();

        service.send(messages::AlterServer {
            name: name.clone(),
            max_memory,
            port,
            up_cmd
        }).await??;

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

type Servers = std::collections::HashMap<String,model::InstanceDescriptor>;

#[Subscription]
impl Subscription {
    async fn servers<'cx>(&self,ctx: &Context<'cx>) -> impl futures::Stream<Item=Servers > + 'cx {
        log::info!("Initializing servers subscription");
        let service = ctx.data_unchecked::<native::Service>();

        tokio_stream::wrappers::IntervalStream::new({
            let mut i = tokio::time::interval(Duration::from_secs(3) + Duration::from_millis(500));
            i.set_missed_tick_behavior(MissedTickBehavior::Skip);
            i
        })
        .then(|_| async {
            //we send heartbeat - can be put out of sync
            service.do_send(messages::Tick);
            //then we ask for the data
            match service.send(messages::Instances {
                f: |i| (i.name.clone(),i.clone())
            }).await {
                Ok(data) => {
                    let data = data
                        .into_iter()
                        .collect::<std::collections::HashMap<String,model::InstanceDescriptor>>();
                    data
                },
                Err(e) => {
                    log::error!("cannot get instance list: {}",e);
                    std::collections::HashMap::new()
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