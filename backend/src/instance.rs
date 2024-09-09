use std::{io::Read, path::Path};

use crate::*;

use anyhow::anyhow;
use std::process::Command;


/// The descriptor of a server
/// at the dir pointed by `place`
/// there should be `msrvDesc.json` file
/// and `run.command` file
pub struct Instance {
    pub desc: model::InstanceDescriptor,
    /// should point at directory where Instance is located
    pub place: Arc<Path>,
    manifest: std::fs::File,
    run_command: std::process::Command,

    process: Option<std::process::Child>
}

impl Instance {

    pub fn load(place: Arc<Path>) -> anyhow::Result<Self> {

        if !place.is_dir() {
            return Err(anyhow!("load should be called on dir"));
        }

        let mut manifest = std::fs::OpenOptions::new()
            .write(true)
            .read(true)
            .open((&*place).join("msrvDesc.json"))?;
        let desc = serde_json::from_reader(&mut manifest)?;

        let mut command_file = std::fs::File::open((&*place).join("run.command"))?;

        let mut run_command = String::new();

        command_file.read_to_string(&mut run_command)?;

        let mut run_command = Command::new(run_command);
        run_command.current_dir(&*place);

        Ok(Self {desc, place, manifest, run_command, process: None})
    }

    pub fn flush(&mut self) {
        serde_json::to_writer(&mut self.manifest, &self.desc);
    }

    pub fn hb(&mut self) {
        todo!()
    }

    pub fn start(&mut self) {
        match self.process {
            Some(_) => return,
            None => {
                if let Ok(ch) = self.run_command.spawn() {
                    self.process = Some(ch)
                }
            }
        }
    }

    pub fn stop(&mut self) {
        todo!()
    }

    pub fn kill(&mut self) {
        match &mut self.process {
            Some(ch) => {
                let _ = ch.kill();
            },
            None => {}
        }
    }
}