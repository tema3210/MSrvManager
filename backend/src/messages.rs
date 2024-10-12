use std::{ffi::OsString, path::Path};

use actix::Message;
use async_graphql::UploadValue;

use crate::*;

/// the central actor messages
pub mod native_messages {
    use std::marker::PhantomData;

    use super::*;

    #[derive(Message,Debug)]
    #[rtype(result = "()")]
    pub struct Stop;


    #[derive(Message,Debug)]
    #[rtype(result = "Option<actix::Addr<A>>")]
    pub struct AddrOf<A: Actor>(pub String,PhantomData<A>);

    impl <A: Actor> AddrOf<A> {
        pub fn new(name: String) -> Self {
            Self(name,PhantomData)
        }
    }

    #[derive(Message,Debug)]
    #[rtype(result = "model::PortsInfo")]
    pub struct Ports;

    #[derive(Message,Debug)]
    #[rtype(result = "anyhow::Result<()>")]
    pub struct Nuke {
        pub who: Arc<Path>
    }

    #[derive(Message)]
    #[rtype(result = "anyhow::Result<()>")]
    pub struct NewServer {
        pub name: String,

        // pub server_jar: PathBuf,
        pub java_args: Vec<OsString>,

        // pub setup_cmd: Option<String>,
        pub url: url::Url,
        pub instance_upload: UploadValue,
        pub max_memory: f64,
        pub port: u16,
        pub rcon: u16
    }

    impl std::fmt::Debug for NewServer {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f
                .debug_struct("NewServer")
                .field("name", &self.name)
                // .field("setup_cmd", &self.setup_cmd)
                .field("url", &self.url)
                .field("instance_upload", &"UploadValue")
                .field("max_memory", &self.max_memory)
                .field("port", &self.port)
                .field("rcon", &self.rcon)
                // .field("server_jar", &self.server_jar)
                .finish()
        }
    }

    #[derive(Message,Debug)]
    #[rtype(result = "anyhow::Result<()>")]
    pub struct DeleteServer {
        pub name: String
    }

    #[derive(Message,Debug)]
    #[rtype(result = "anyhow::Result<()>")]
    pub struct AlterServer {
        pub name: String,
        pub msg: super::instance_messages::AlterServer
    }

    #[derive(Message,Debug)]
    #[rtype(result = "Vec<O>")]
    pub struct Instances<O,F>
        where
            O: Send + 'static,
            F: Send + Sync + Fn(&instance::Instance) -> Option<O> + 'static,
    {
        pub f: F,
}


}

/// instance actor messages
pub mod instance_messages {
    use super::*;

    #[derive(Message,Debug)]
    #[rtype(result = "anyhow::Result<()>")]
    pub struct SwitchServer {
        pub should_run: bool
    }

    #[derive(Message,Debug)]
    #[rtype(result = "anyhow::Result<()>")]
    pub struct Kill;

    #[derive(Message,Debug)]
    #[rtype(result = "anyhow::Result<()>")]
    pub struct AlterServer {
        pub max_memory: Option<f64>,
        pub port: Option<u16>,
        pub java_args: Option<Vec<OsString>>,
    }

    #[derive(Message,Debug)]
    #[rtype(result = "Option<O>")]
    pub struct Instance<O,F>
        where
            O: Send + 'static,
            F: Send + Sync + Fn(&instance::Instance) -> Option<O> + 'static,
    {
        pub f: F,
    }
}


#[derive(Message,Debug)]
#[rtype(result = "()")]
pub struct Tick;

