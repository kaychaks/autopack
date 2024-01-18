mod proc_file;

use super::log::trying;
use crate::{error::AppError, log::success};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
};
use tracing::{debug, warn};

pub(crate) use proc_file::ProcFile;

/// Runtime information of autopack
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct Runtime {
    /// `.autopack` directory path
    dir: PathBuf,
    /// proc file
    proc_file: ProcFile,
}

pub(crate) struct RuntimeBuilder {
    dir: PathBuf,
    proc_file: Option<ProcFile>,
}

impl Default for Runtime {
    fn default() -> Self {
        Runtime {
            dir: PathBuf::new().join(".autopack"),
            proc_file: ProcFile::default(),
        }
    }
}

fn create_runtime_dir(path: &Path, force_create: bool) -> Result<PathBuf, io::Error> {
    let rtp = |created: bool| {
        let p = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        if created {
            debug!("Created runtime folder at {:?}", p);
            success("Created runtime folder");
        }else {
            debug!("Runtime folder already present at {:?}", p);
            success("Runtime folder already present");
        }

        p
    };
    if !path.exists() {
        fs::create_dir(path)?;
        Ok(rtp(true))
    } else if force_create {
        let full_path = path.canonicalize()?;
        warn!(
            "runtime folder present at {:?}; removing the same",
            full_path
        );
        fs::remove_dir_all(path)?;
        create_runtime_dir(path, false)
    } else {
        Ok(rtp(false))
    }
}

impl Runtime {
    pub(crate) fn builder(mod_path: &Path) -> RuntimeBuilder {
        trying("Generating auto-pack runtime");
        RuntimeBuilder {
            dir: mod_path.join(".autopack"),
            proc_file: None,
        }
    }

    pub(crate) fn dir(&self) -> PathBuf {
        self.dir.clone()
    }

    pub(crate) fn proc_file(&self) -> ProcFile {
        self.proc_file.clone()
    }

    pub(crate) fn project_toml(&self) -> String {
        self.dir
            .join("project.toml")
            .to_str()
            .expect("Expected a valid project.toml file")
            .to_string()
    }
}

impl RuntimeBuilder {
    pub(crate) fn dir(&mut self, force_create_runtime: bool) -> Result<&mut Self, AppError> {
        trying("Creating runtime folder");
        let runtime_full_path = create_runtime_dir(self.dir.as_path(), force_create_runtime)
            .map_err(|e| AppError::IOError("Failed to create runtime dir", e))?;

        self.dir = runtime_full_path;
        Ok(self)
    }

    pub(crate) fn proc_file(&mut self, live_reload: bool) -> Result<&mut Self, AppError> {
        trying("Creating proc file");

        let proc_file = ProcFile::builder()
            .override_start_entry(live_reload)
            .export(&self.dir)
            .map_err(|e| AppError::PostConfigureError("Failed creating Procfile", e))?
            .build();

        self.proc_file = Some(proc_file.clone());
        debug!("Created proc file at {:?}", proc_file.proc_file_path());
        success("Created proc file");
        Ok(self)
    }

    pub(crate) fn build(&mut self) -> Runtime {
        success("Generated auto-pack runtime");
        Runtime {
            dir: self.dir.clone(),
            proc_file: self
                .proc_file
                .clone()
                .unwrap_or(Runtime::default().proc_file),
        }
    }
}
