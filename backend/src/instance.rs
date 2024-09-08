use std::path::Path;

use crate::*;

use anyhow::anyhow;


pub struct Instance {
    pub desc: model::InstanceDesc,
    pub place: Arc<Path>,
    manifest: std::fs::File,

    process: Option<std::process::Child>
}

impl Instance {

    pub fn load(place: Arc<Path>) -> Option<Self> {

        let manifest = (&*place).join("msrvInfo.json");

        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .read(true)
            .open(manifest).ok()?;
        let desc = serde_json::from_reader(&mut file).ok()?;

        Some(Self {desc, place, manifest: file, process: None})
    }

    pub fn flush(&mut self) {
        serde_json::to_writer(&mut self.manifest, &self.desc);
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