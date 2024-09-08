use std::path::Path;

use crate::*;

use anyhow::anyhow;

// pub fn patch_config(config: &mut model::InstanceDesc, change: model::ServerChange) -> Result<(),anyhow::Error> {
//     let mut file = std::fs::File::options()
//         .read(false)
//         .write(true)
//         .open(&*config.place)?;

//     match change {
//         model::ServerChange::MaxMemory(mm) => {
//             config.max_memory = mm;
//         },
//         model::ServerChange::Port(_) => todo!(),
//         model::ServerChange::Rcon(_) => todo!(),
//         model::ServerChange::Run(_) => todo!(),
//     }

// }


pub struct Instance {
    pub desc: model::InstanceDesc,
    pub place: Arc<Path>,
    file: std::fs::File,

    process: Option<std::process::Child>
}

impl Instance {
    pub fn load(from: Arc<Path>) -> Option<Self> {
        todo!()
    }

    pub fn hb(&mut self) {
        todo!()
    }

    pub fn start(&mut self) {
        todo!()
    }

    pub fn stop(&mut self) {
        todo!()
    }

    pub fn kill(&mut self) {
        todo!()
    }
}