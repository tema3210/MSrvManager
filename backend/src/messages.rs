use std::path::Path;

use actix::Message;

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
    pub struct InitServer<P: Send + 'static> {

        // pub server_jar: PathBuf,
        pub java_args: Vec<String>,

        // pub setup_cmd: Option<String>,
        pub url: url::Url,
        // pub instance_upload: UploadValue,
        pub max_memory: f64,
        pub ports: model::Ports,

        pub ext: P
    }

    impl<P: Send + 'static + std::fmt::Debug > std::fmt::Debug for InitServer<P> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f
                .debug_struct("NewServer")
                // .field("setup_cmd", &self.setup_cmd)
                .field("url", &self.url)
                .field("instance_upload", &"UploadValue")
                .field("max_memory", &self.max_memory)
                .field("ports", &self.ports)
                // .field("server_jar", &self.server_jar)
                .field("ext", &self.ext)
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

    #[derive(Message,Debug)]
    #[rtype(result = "Vec<String>")]
    pub struct Broken;

    #[derive(Message,Debug)]
    #[rtype(result = "Option<serde_json::Value>")]
    pub struct DataOfBroken {
        pub name: String
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
        pub java_args: Option<Vec<String>>,
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

