use anyhow::anyhow;
use async_graphql::UploadValue;
use zip::ZipArchive;

use crate::*;

use std::{ffi::OsString, fs::{File, Permissions}, ops::Range, path::Path, process::Command};

#[derive(Debug)]
pub struct Indices(Range<u16>, bit_set::BitSet);

impl Indices {
    pub fn new(r: Range<u16>) -> Self {
        Self(r, bit_set::BitSet::new())
    }

    pub fn range(&self) -> Range<u16> {
        self.0.clone()
    }

    pub fn try_take(&mut self, idx: u16) -> Result<(), anyhow::Error> {
        if !self.0.contains(&idx) {
            return Err(anyhow!("out of bounds"));
        };

        if self.1.contains(idx.into()) {
            return Err(anyhow!("already occupied"));
        };

        self.1.insert(idx.into());

        Ok(())
    }

    pub fn free(&mut self, idx: u16) -> anyhow::Result<()> {
        if !self.0.contains(&idx) {
            return Err(anyhow!("out of bounds"));
        };

        if self.1.contains(idx.into()) {
            self.1.remove(idx.into());
            return Ok(());
        };

        Err(anyhow!("already freed"))
    }

    /// iterate over taken ports
    pub fn taken(&self) -> Vec<u16> {
        self.1
            .iter()
            .map(|p| p.try_into().expect("have port larger than it should be"))
            .collect()
    }
}

/// it only disposed process, not kills its
pub fn dispose(mut child: std::process::Child) {
    std::thread::spawn(move || {
        let r = child.wait();
        match r {
            Ok(status) => {
                log::info!("child with pid {} died with status: {:?}", child.id(), status);
            }
            Err(e) => {
                log::error!("error while waiting for child to die: {}", e);
            }
        }
    });
}

/// make sure that dir exists and has manifest file
pub fn initialize_server_directory<P: AsRef<Path>,R>(at: P,act: impl FnOnce() -> anyhow::Result<R>) -> anyhow::Result<R> {
    log::info!("preparing dir for server at {:?}",at.as_ref());

    std::fs::create_dir(at.as_ref()).unwrap();

    let r = act()?;

    std::fs::File::create_new(at.as_ref().join(instance::MANIFEST_NAME))?;

    Ok(r)
}

pub fn open_manifest<P: AsRef<Path>>(at: P) -> Result<File, std::io::Error> {
    File::options()
        .write(true)    
        .read(true)
        .open(at.as_ref().join(instance::MANIFEST_NAME))
}

pub fn generate_classpath<P: AsRef<Path>>(at: P) -> anyhow::Result<OsString> {

    let separator = if cfg!(target_os = "windows") { ";" } else { ":" };

    std::fs::read_dir(at.as_ref())?
        .filter_map(|e| e.ok())
        .try_fold(OsString::new(),|mut cp,e| {

            // these we don't preload
            if e.path().ends_with("net/minecraft") {
                return Ok(cp)
            }

            if e.path().is_dir() {
                let icp = generate_classpath(at.as_ref())?;
                cp.push(icp);
                cp.push(separator);
                return Ok(cp)
            }

            if e.path().extension().map(|e| e == "jar").unwrap_or(false) {
                cp.push(&e.path());
                cp.push(separator);
                return Ok(cp)
            };

            Ok(cp)
        })
}


/// this blocks thread
pub fn unpack_at(at: impl AsRef<Path>, data: &mut UploadValue) -> anyhow::Result<()> {

    let mut archive = ZipArchive::new(&mut data.content)?;

    log::info!("starting to unpack at {:?}", at.as_ref());

    for i in 0..archive.len() {
        let mut archive_file = archive.by_index(i)?;

        let outpath = match archive_file.enclosed_name() {
            Some(path) => at.as_ref().join(path),
            None => continue,
        };

        // Create directories if necessary
        if archive_file.is_dir() {
            log::trace!("creating dir {:?}", &*outpath);
            let _ = std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                log::trace!("create dir all at {:?} for {:?}", p, &outpath);
                let _ = std::fs::create_dir_all(p)?;
            }
            log::trace!("making {:?}", &*outpath);
            let mut outfile = File::create(&outpath)?;
            log::trace!("copying {:?}", &*outpath);

            std::io::copy(&mut archive_file, &mut outfile)?;

            // Set file permissions
            if let Some(mode) = archive_file.unix_mode() {
                let permissions = <Permissions as std::os::unix::fs::PermissionsExt>::from_mode(mode);
                std::fs::set_permissions(&outpath, permissions.clone())?;
                log::trace!("set permissions {:?} for {:?}", permissions, &*outpath);
            }

            log::info!("copied {:?}", &*outpath);
        }
    };
    Ok(())
}

pub fn make_command(c: impl AsRef<str>) -> std::process::Command {
    let pts: Vec<_> = c.as_ref().split_whitespace().collect();

    let mut c = Command::new(pts[0]);
    c.args(pts[1..].iter());
    c
}

pub fn patch_server_props(
    at: impl AsRef<Path>,
    port: u16,
    rcon: u16,
    max_memory: usize,
    password: &str
) -> anyhow::Result<()> { 

    let output = Command::new("sh")
        .arg(instance::PATCH_SH_PATH)
        .env("MPORT", port.to_string())
        .env("MRCON", rcon.to_string())
        .env("MAXMEMORY", format!("{}G", max_memory))
        .env("PROPERTIES_FILE", at.as_ref().join(instance::SERVER_PROPERTIES_FILE))
        .env("PASSWORD", password)
        .output()?;

    if !output.status.success() {
        Err(anyhow!(
            "patch_server_props script failed with status: {}",
            output.status
        ))
    } else {
        log::trace!("patch_server_props script executed successfully");
        Ok(())
    }
}