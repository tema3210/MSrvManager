use async_graphql::{Schema, Subscription, Upload };

use async_graphql::{Context, Object};
use futures::StreamExt;
use tokio::time::MissedTickBehavior;

use crate::*;

pub struct Query;

#[Object]
impl Query {
    async fn app_version(&self) -> &'static str {
        "1.1"
    }

    async fn ports_taken<'cx>(&self, ctx: &Context<'cx>) -> anyhow::Result<messages::PortsInfo> {
        let service = ctx.data_unchecked::<native::Service>();
        Ok(service.send(messages::Ports).await?)
    }

    async fn rcons<'cx>(&self, ctx: &Context<'cx>) -> Vec<serde_json::Value> {
        let service = ctx.data_unchecked::<native::Service>();
        match service.send(messages::Instances {
            f: |i| Some(serde_json::json!({
                "name": i.desc.name,
                "rcon": i.desc.rcon
            })),
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
    /// path to jar of server inside of upload
    server_jar: String,

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
        
        service.send(messages::SwitchServer {
            name,
            should_run
        }).await??;
        Ok(true)
    }

    async fn new_server<'cx>(
        &self,
        ctx: &Context<'cx>,
        data: NewServer,
        password: String
    ) -> Result<bool,anyhow::Error> {
        let service = ctx.data_unchecked::<native::Service>();

        let pass = ctx.data_unchecked::<Password>();

        if password != pass.0 {
            log::error!("wrong password: {}",password);
            return Err(anyhow::anyhow!("wrong password"));
        }

        let server_jar: PathBuf = data.server_jar.parse()?;

        if !server_jar.ends_with(".jar") {
            return Err(anyhow::anyhow!("server_jar must be a path to a .jar file"));
        }

        if server_jar.is_absolute() {
            return Err(anyhow::anyhow!("server_jar must be a relative path"));
        }

        let val = data.instance_upload.value(ctx)?;
        
        service.send(messages::NewServer {
            name: data.name,
            up_cmd: data.up_cmd,
            server_jar,
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
        password: String
    ) -> Result<bool,anyhow::Error> {

        let pass = ctx.data_unchecked::<Password>();

        if password != pass.0 {
            log::error!("wrong password: {}",password);
            return Err(anyhow::anyhow!("wrong password"));
        }

        let service = ctx.data_unchecked::<native::Service>();

        service.send(messages::AlterServer {
            name: name.clone(),
            max_memory,
            port,
            up_cmd
        }).await??;

        Ok(true)
    }

    async fn delete_server<'cx>(&self,ctx: &Context<'cx>,name: String, password: String) -> Result<bool,anyhow::Error> {
        let service = ctx.data_unchecked::<native::Service>();

        let pass = ctx.data_unchecked::<Password>();

        if pass.0 != password {
            log::error!("wrong password: {}",password);
            return Err(anyhow::anyhow!("wrong password"));
        }

        service.send(messages::DeleteServer {
            name
        }).await??;
        Ok(true)
    }
}

pub struct Subscription;

type Servers = std::collections::HashMap<String,serde_json::Value>;

#[Subscription]
impl Subscription {
    async fn servers<'cx>(&self,ctx: &Context<'cx>) -> impl futures::Stream<Item=Servers> + 'cx {
        let service = ctx.data_unchecked::<native::Service>();

        tokio_stream::wrappers::IntervalStream::new({
            let mut i = tokio::time::interval(Duration::from_secs(3) + Duration::from_millis(500));
            i.set_missed_tick_behavior(MissedTickBehavior::Skip);
            i
        })
        .then(|_| async {
            //then we ask for the data
            match service.send(messages::Instances {
                f: |i| Some((i.desc.name.clone(),i.desc.clone(),i.state))
            }).await {
                Ok(data) => {
                    let data = data
                        .into_iter()
                        .map(|(name,val,state)| {
                            let res = match state {
                                instance::InstanceState::Normal => {
                                    serde_json::json!({
                                        "data": val,
                                        "state": "normal"
                                    })
                                },
                                instance::InstanceState::Downloading => {
                                    serde_json::json!({
                                        "data": null,
                                        "state": "downloading"
                                    })
                                },
                                instance::InstanceState::Stopping => {
                                    serde_json::json!({
                                        "data": null,
                                        "state": "stopping"
                                    })
                                },
                            
                            };
                            (name,res)
                        })
                        .collect::<std::collections::HashMap<String,serde_json::Value>>();
                    data
                },
                Err(e) => {
                    log::error!("cannot get instance list: {}",e);
                    std::collections::HashMap::new()
                }
            }
        })
    }

    async fn instance<'cx>(&self,ctx: &Context<'cx>,name: String) -> impl futures::Stream<Item=Option<serde_json::Value>> + 'cx {
        let service = ctx.data_unchecked::<native::Service>();

        let name: Arc<str> = Arc::from(name.as_str());

        tokio_stream::wrappers::IntervalStream::new({
            let mut i = tokio::time::interval(Duration::from_secs(2));
            i.set_missed_tick_behavior(MissedTickBehavior::Skip);
            i
        })
        .map(move |_| Arc::clone(&name) )
        .then({
            move |name| async move {

                //then we ask for the data
                let data = service.send(messages::Instance {
                    name: Arc::clone(&name),
                    f: |i| Some(i.desc.clone())
                }).await;

                match data {
                    Ok(Some(data)) => {
                        Some(serde_json::to_value(data).unwrap())
                    },
                    Ok(None) => {
                        None
                    },
                    Err(e) => {
                        log::error!("cannot get instance {}: {}",name,e);
                        None
                    }
                }
            }
        })
    }
}


// A root schema consists of a query and a mutation.
// Request queries can be executed against a RootNode.
pub type SrvsSchema = Schema<Query, Mutation, Subscription>;

struct Password(String);

pub fn schema(addr: crate::native::Service,pass: String) -> SrvsSchema {
    Schema::build(Query,Mutation, Subscription)
    .data::<native::Service>(addr)
    .data(Password(pass))
    .finish()
}