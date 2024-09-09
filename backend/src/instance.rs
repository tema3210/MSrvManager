use std::{io::{Read, Write}, path::Path};

use crate::*;

use anyhow::anyhow;
use std::process::Command;


/// The descriptor of a server
/// at the dir pointed by `place`
/// there should be `msrvDesc.json` file
/// and `run.command` file
pub struct Instance {
    pub desc: model::InstanceDescriptor,
    pub is_downloading: bool,
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
        let desc: model::InstanceDescriptor = model::InstanceDescriptor::from_file(&mut manifest)?;

        let mut command_file = std::fs::File::open((&*place).join("run.command"))?;

        let mut run_command = String::new();

        command_file.read_to_string(&mut run_command)?;

        let mut run_command = Command::new(run_command);
        run_command.current_dir(&*place);

        Ok(Self {desc, place, manifest, run_command, process: None, is_downloading: false})
    }

    pub fn flush(&mut self) {
        serde_json::to_writer(&mut self.manifest, &self.desc);
    }

    pub fn hb(&mut self) {
        if self.is_downloading {
            return
        }
        match &mut self.process {
            Some(ch) => {
                if let Ok(process) = procfs::process::Process::new(ch.id().try_into().unwrap()) {
                    let Ok(status) = process.status() else {
                        return;
                    };
                    // memory in KB
                    if let Some(memory) = status.vmrss {
                        let memory = (memory / (1024 * 1024)) as f64;
                        self.desc.memory = Some(memory)
                    }
                } else {
                    if self.desc.state == model::ServerState::Running {
                        self.desc.state = model::ServerState::Crashed
                    }
                }
            }, 
            None => {}
        }
        self.flush()
    }

    pub fn start(&mut self) {
        if self.is_downloading {
            return
        }
        match self.process {
            Some(_) => return,
            None => {
                if let Ok(ch) = self.run_command.spawn() {
                    self.process = Some(ch)
                };
                self.hb()
            }
        };
    }

    pub fn stop(&mut self) {
        if self.is_downloading {
            return
        }
        match &mut self.process {
            Some(ch) => {
                if let Some(pipe) =  &mut ch.stdin {
                    let res = pipe.write(b"stop\n");
                    if res.is_err() || matches!(res,Ok(0)) {
                        // we should wait here for like 5 secs
                    }
                };
            },
            None => {}
        };
        self.desc.state = model::ServerState::Stopped;
    }

    pub fn kill(&mut self) {
        match &mut self.process {
            Some(ch) => {
                ch.kill();
                self.desc.state = model::ServerState::Stopped;
                self.desc.memory = None
            },
            None => {}
        }
    }
}