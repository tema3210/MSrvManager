use std::collections::VecDeque;
use std::pin::Pin;

use async_graphql::{Schema, Subscription, Upload };

use async_graphql::{Context, Object};
use futures::{Stream, StreamExt};
use tokio::time::MissedTickBehavior;

use crate::*;

use crate::messages::{instance_messages, native_messages};

pub struct Query;

#[Object]
impl Query {
    async fn app_version(&self) -> &'static str {
        "1.1"
    }

    async fn ports_taken<'cx>(&self, ctx: &Context<'cx>) -> anyhow::Result<model::PortsInfo> {
        let service = ctx.data_unchecked::<native::Service>();
        Ok(service.send(native_messages::Ports).await?)
    }

    //todo: add here names someday
    async fn rcons<'cx>(&self, ctx: &Context<'cx>) -> serde_json::Value {
        let service = ctx.data_unchecked::<native::Service>();
        match service.send(native_messages::Ports).await {
            Ok(info) => serde_json::json!({
                "rcons": info.rcons,
            }),
            Err(_) => serde_json::json!({
                "rcons": []
            })
        }
    }
}

pub struct Mutation;

#[derive(async_graphql::InputObject)]
pub struct NewServer {
    name: String,

    /// path to jar of server inside of upload
    server_jar: String,
    java_args: String,

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

        let addr = service.send(
            native_messages::AddrOf::new(name.clone())
        ).await?;
        
        if let Some(addr) = addr {
            addr.send(instance_messages::SwitchServer {
                should_run
            }).await??;
            return Ok(true)
        } else {
            return Ok(false)
        }
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

        match server_jar.extension() {
            Some(ext) => {
                if ext != ".jar" {
                    return Err(anyhow::anyhow!("server_jar must be a path to a .jar file"));
                }
            },
            None => {
                return Err(anyhow::anyhow!("server_jar must be a path to a .jar file"));
            }
        }

        if server_jar.is_absolute() {
            return Err(anyhow::anyhow!("server_jar must be a relative path"));
        }

        let val = data.instance_upload.value(ctx)?;

        let args = data.java_args.split_whitespace().map(|s| s.into()).collect::<Vec<_>>();
        
        service.send(native_messages::NewServer {
            name: data.name,
            server_jar,
            setup_cmd: data.setup_cmd,
            url: data.url,
            max_memory: data.max_memory,
            port: data.port,
            rcon: data.rcon,
            instance_upload: val,
            java_args: args
        }).await??;

        Ok(true)

    }

    async fn alter_server<'cx>(
        &self,
        ctx: &Context<'cx>,
        name: String,
        max_memory: Option<f64>,
        java_args: Option<String>,
        port: Option<u16>,
        password: String
    ) -> Result<bool,anyhow::Error> {

        let pass = ctx.data_unchecked::<Password>();

        if password != pass.0 {
            log::error!("wrong password: {}",password);
            return Err(anyhow::anyhow!("wrong password"));
        }

        let service = ctx.data_unchecked::<native::Service>();

        service.send(native_messages::AlterServer {
            name: name.clone(),
            msg: instance_messages::AlterServer {
                max_memory,
                java_args: java_args.map(|s| s.split_whitespace().map(|s| s.into()).collect::<Vec<_>>()),
                port
            }
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

        service.send(native_messages::DeleteServer {
            name
        }).await??;
        Ok(true)
    }

    async fn rcon_message<'cx>(&self,ctx: &Context<'cx>,name: String, message: String, password: String) -> Result<bool,anyhow::Error> {
        let service = ctx.data_unchecked::<native::Service>();

        let pass = ctx.data_unchecked::<Password>();

        if pass.0 != password {
            log::error!("wrong password: {}",password);
            return Err(anyhow::anyhow!("wrong password"));
        }

        let Some(addr) = service.send(
            native_messages::AddrOf::new(name.clone())
        ).await? else {
            return Err(anyhow::anyhow!("no such server: {}",name));
        };

        addr.send(rcon::RconMessage {
            cmd: message
        }).await??;

        Ok(true)
    }
}

pub struct Subscription;

type Servers = std::collections::HashMap<String,serde_json::Value>;

type RconStream = Pin<Box<dyn Stream<Item = Vec<String>> + Send + 'static>>;

const WINDOW_SIZE: usize = 12;

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
            match service.send(native_messages::Instances {
                f: |i| Some((
                    i.desc().cloned(),
                    i.state(),
                    i.name()
                ))
            }).await {
                Ok(data) => {
                    let data = data
                        .into_iter()
                        .map(|(desc,state,place)| {
                            (
                                place,
                                serde_json::json!({
                                    "data": desc,
                                    "state": state
                                })
                            )
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

    async fn instance<'cx>(&self,ctx: &Context<'cx>,name: String) -> anyhow::Result<impl futures::Stream<Item=Option<serde_json::Value>> + 'cx> {
        let service = ctx.data_unchecked::<native::Service>();

        let Some(addr) = service.send(
            native_messages::AddrOf::new(name.clone())
        ).await? else {
            return Err(anyhow::anyhow!("no such server: {}",name));
        };

        let stream = tokio_stream::wrappers::IntervalStream::new({
            let mut i = tokio::time::interval(Duration::from_secs(2));
            i.set_missed_tick_behavior(MissedTickBehavior::Skip);
            i
        })
        .map( move |_| addr.clone() )
        .then({
            move |addr| async move {

                //then we ask for the data
                let data = addr.send(instance_messages::Instance {
                    f: |i| i.desc().cloned()
                }).await;

                match data {
                    Ok(Some(data)) => {
                        Some(serde_json::to_value(data).unwrap())
                    },
                    Ok(None) => {
                        None
                    },
                    Err(e) => {
                        log::error!("cannot get instance: {:}",e);
                        None
                    }
                }
            }
        });

        Ok(stream)
    }

    async fn rcon_output<'cx>(&self, ctx: &Context<'cx>, name: String) -> anyhow::Result<RconStream> {
        let service = ctx.data_unchecked::<native::Service>();

        let Some(addr) = service.send(
            native_messages::AddrOf::new(name.clone())
        ).await? else {
            return Err(anyhow::anyhow!("no such server: {}",name));
        };

        let stream = addr.send(rcon::RconSubscription).await??;

        let mut window = VecDeque::with_capacity(WINDOW_SIZE);

        let stream = stream
            .map(move |i| (i,addr.clone()))
            //rewrite with unfold to terminate subscription
            .filter_map(|(out,addr)| async move {
                match out {
                    rcon::RconOutput::CommandResponse(resp) => {
                        Some(resp)
                    },
                    rcon::RconOutput::Error(error) => {
                        log::error!("rcon error: {}",error);
                        None
                    },
                    rcon::RconOutput::ConnectionClosed => {
                        addr.send(rcon::RconDown).await.unwrap();
                        None
                    },
                }
            })
            .map(move |msg| {
                
                window.push_back(msg);

                if window.len() == WINDOW_SIZE + 1 {
                    window.pop_front();
                }

                let dump = window.iter().cloned().collect();
                dump
            })
            .boxed();

        Ok(stream)
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