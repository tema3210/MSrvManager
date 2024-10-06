use std::{
    collections::HashMap,
    ops::Range,
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use futures::{stream::FuturesUnordered, StreamExt};
use instance::Instance;

use crate::*;
use crate::messages::{native_messages,instance_messages};
use utils::Indices;

use actix::prelude::*;


#[derive(Clone,Debug)]
pub struct Server {
    addr: Addr<instance::Instance>,
    ports: model::Ports,
}

pub struct Servers {
    servers_dir: PathBuf,
    rcon_range: Indices,
    port_range: Indices,
    timeout: Duration,
    password: String,

    servers: HashMap<std::sync::Arc<Path>, Server>,
}

impl Actor for Servers {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let Ok(servers) = std::fs::read_dir(&self.servers_dir) else {
            log::error!("couldn't read servers dir");
            ctx.stop();
            return;
        };
        servers.filter_map(|de| {
            let de = de.ok()?;
            if de.path().is_dir() {
                Some(de)
            } else {
                None
            }
        }).filter_map(|at| {
            let arc_path: Arc<Path> = at.path().into();
            let env = instance::InstanceEnv {
                timeout: self.timeout,
                servers: ctx.address(),
                password: self.password.clone(),
            };
            match instance::Instance::load(Arc::clone(&arc_path),env) {
                Ok((instance,ports)) => {

                    if let Err(e) = self.port_range.try_take(ports.port) {
                        log::error!("a servers {:?} port {} is taken", at.path(), e);
                        return None;
                    }

                    if let Err(e) = self.rcon_range.try_take(ports.rcon) {
                        log::error!("a servers {:?} rcon port {} is taken", at.path(), e);
                        return None;
                    }

                    Some((arc_path.clone(), instance, ports))
                },
                Err(e) => {
                    log::error!("couldn't load server at {:?} due to {:?}, nuking", &arc_path, e);
                    let _ = std::fs::remove_dir_all(arc_path.as_ref());
                    None
                }
            }
        }).for_each(|(arc_path, instance,ports)| {
            self.servers.insert(arc_path, Server {
                addr: instance.start(),
                ports
            });
        });
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        log::info!("stopping servers");
        let stop_futures = self.servers.values().map(|srv| {
            srv.addr.send(instance_messages::SwitchServer {should_run: false})
        }).collect::<FuturesUnordered<_>>();

        let stop = stop_futures.collect::<Vec<_>>().into_actor(self).then(|res, _, _| {
            for i in res {
                if let Err(e) = i {
                    log::error!("failed to stop server: {:?}", e);
                }
            };
            fut::ready(())
        });
        ctx.wait(stop);
        Running::Stop
    }
}

impl Handler<native_messages::Stop> for Servers {
    type Result = ();

    fn handle(&mut self, _: native_messages::Stop, cx: &mut Self::Context) -> Self::Result {
        cx.stop();
        ()
    }
}

impl Servers {
    fn name_to_path<P: AsRef<Path>>(&self, name: P) -> PathBuf {
        self.servers_dir.as_path().join(name)
    }

    pub fn new<P: AsRef<Path>>(
        path: P,
        rcon_range: Range<u16>,
        port_range: Range<u16>,
        timeout: Duration,
        password: String,
    ) -> Self {
        let servers_dir = path.as_ref().to_owned();

        let rcon_range = Indices::new(
            rcon_range.clone()
        );
        let port_range = Indices::new(port_range);

        return Self {
            servers_dir,
            rcon_range,
            port_range,
            servers: HashMap::new(),
            timeout,
            password
        };
        
    }

    fn hb(&mut self) {
        for (_, i) in &mut self.servers {
            i.addr.do_send(messages::Tick);
        }
    }

    fn nuke(&mut self, who: impl AsRef<Path>) -> anyhow::Result<()> {
        let path = who.as_ref();
        if let Some(server) = self.servers.remove(path.into()) {
            self.port_range.free(server.ports.port)?;
            self.rcon_range.free(server.ports.rcon)?;
        }
        std::fs::remove_dir_all(path)?;
        Ok(())
    }
}

pub type Service = actix::Addr<Servers>;

impl Handler<native_messages::AddrOf<instance::Instance>> for Servers {
    type Result = Option<Addr<instance::Instance>>;

    fn handle(&mut self, msg: native_messages::AddrOf<instance::Instance>, _: &mut Self::Context) -> Self::Result {
        let name = self.name_to_path(msg.0);
        self.servers.get_mut::<Path>(name.as_ref()).map(|s| s.addr.clone())
    }
}

impl Handler<native_messages::Ports> for Servers {
    type Result = MessageResult<native_messages::Ports>;

    fn handle(&mut self, _: native_messages::Ports, _: &mut Self::Context) -> Self::Result {
        let pr = self.port_range.range();
        let rr = self.rcon_range.range();
        MessageResult(model::PortsInfo {
            ports: self.port_range.taken(),
            rcons: self.rcon_range.taken(),
            port_limits: [pr.start, pr.end],
            rcon_limits: [rr.start, rr.end],
        })
    }
}

impl<O, F> Handler<native_messages::Instances<O, F>> for Servers
where
    F: Send + Sync + Fn(&instance::Instance) -> Option<O> + 'static,
    O: Send + 'static,
{
    type Result = ResponseFuture<Vec<O>>;

    fn handle(&mut self, m: native_messages::Instances<O, F>, _: &mut Context<Self>) -> Self::Result {

        let f = Arc::new(m.f);

        let summary = self
            .servers
            .values()
            .map(|Server {addr, ..}| addr.clone())
            .map(|addr| {
                let f = Arc::clone(&f);
                addr.send(instance_messages::Instance {
                    f: move |i| (f)(i),
                })
            })
            .collect::<FuturesUnordered<_>>();

        Box::pin(async move {
            summary.fold(Vec::new(), |mut acc, o| async {
                match o {
                    Ok(Some(o)) => acc.push(o),
                    Ok(None) => {}
                    Err(e) => {
                        log::error!("error while getting instance: {:?}", e);
                    }
                };
                acc
            }).await
        })
    }
}

impl Handler<native_messages::NewServer> for Servers {
    type Result = anyhow::Result<()>;

    fn handle(&mut self, msg: native_messages::NewServer, ctx: &mut Self::Context) -> Self::Result {
        let path = self.name_to_path(&msg.name);

        if path.exists() && self.servers.contains_key((&*path).into()) {
            return Err(anyhow!("server name is already in use"));
        }

        log::trace!("creating server: {:?}", &msg);

        match (
            self.rcon_range.try_take(msg.rcon),
            self.port_range.try_take(msg.port),
        ) {
            //rcon   port
            (Ok(()), Ok(())) => {}
            (Ok(()), Err(e)) => {
                self.rcon_range.free(msg.rcon).unwrap(); // free rcon back
                return Err(e);
            }
            (Err(e), Ok(())) => {
                self.port_range.free(msg.port).unwrap(); // free port back
                return Err(e);
            }
            (Err(_), Err(_)) => return Err(anyhow!("both rcon and port are bad")),
        };

        log::info!("create server at {:?}", &*path);

        let desc: model::InstanceDescriptor = model::InstanceDescriptor {
            server_jar: msg.server_jar,
            name: msg.name,
            mods: msg.url,
            max_memory: msg.max_memory,
            memory: None,
            port: msg.port,
            rcon: msg.rcon,
            java_args: msg.java_args,
        };

        let instance_place: Arc<Path> = path.into();

        let instance = Instance::create(
            Arc::clone(&instance_place),
            desc,
            msg.setup_cmd,
            msg.instance_upload,
            instance::InstanceEnv { 
                servers: ctx.address(),
                timeout: self.timeout,
                password: self.password.clone(),
            },
        ).start();

        self.servers.insert(instance_place, Server {
            addr: instance,
            ports: model::Ports {
                port: msg.port,
                rcon: msg.rcon,
            }
        });

        Ok(())
    }
}

impl Handler<native_messages::AlterServer> for Servers {
    type Result = anyhow::Result<()>;

    fn handle(&mut self, msg: native_messages::AlterServer, _: &mut Self::Context) -> Self::Result {
        let path = self.name_to_path(msg.name);

        let Some(srv) = self.servers.get_mut::<Path>(path.as_ref()) else {
            return Err(anyhow!("server not found"));
        };

        if let Some(port) = msg.msg.port {
            if srv.ports.port != port {
                self.port_range.try_take(port)?;
                self.port_range.free(srv.ports.port)?;
                srv.ports.port = port;
            }
        }

        //we don't really have to check for success here
        let _ = srv.addr.send(msg.msg);
        
        Ok(())
    }
}


impl Handler<messages::Tick> for Servers {
    type Result = MessageResult<messages::Tick>;

    fn handle(&mut self, _: messages::Tick, _: &mut Self::Context) -> Self::Result {
        log::trace!("tick tac");
        self.hb();
        MessageResult(())
    }
}

impl Handler<native_messages::DeleteServer> for Servers {
    type Result = anyhow::Result<()>;

    fn handle(&mut self, msg: native_messages::DeleteServer, _: &mut Self::Context) -> Self::Result {
        let path = self.name_to_path(msg.name);

        let Some((path,srv)) = self.servers.remove_entry::<Path>(path.as_ref()) else {
            return Err(anyhow!("server not found"));
        };

        self.port_range.free(srv.ports.port)?;
        self.rcon_range.free(srv.ports.rcon)?;

        let _ = srv.addr.send(instance_messages::Kill);

        self.nuke(path)?;

        Ok(())
    
    }
}

impl Handler<native_messages::Nuke> for Servers {
    type Result = anyhow::Result<()>;

    fn handle(&mut self, msg: native_messages::Nuke, _: &mut Self::Context) -> Self::Result {
        self.nuke(msg.who)
    }
}
