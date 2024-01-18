mod crypto;
mod filesystem;
#[cfg(test)]
mod tests;

use self::filesystem::StateFiles;
use super::{pack::Pack, runtime::Runtime};
use crate::{
    buildpack::BuildPackProject,
    docker::Docker,
    error::AppError,
    log::{banner, trying},
    package_json::Project,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::debug;

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Clone)]
pub(crate) struct AutoPack {
    pub(crate) runtime: Runtime,
    pub(crate) pack_cli: Pack,
    pub(crate) docker: Docker,
    pub(crate) buildpack: BuildPackProject,
    pub(crate) client_project: Project,
}

impl AutoPack {
    #[cfg(test)]
    pub fn new(runtime: Option<Runtime>) -> AutoPack {
        AutoPack {
            runtime: runtime.unwrap_or_default(),
            pack_cli: Pack::default(),
            docker: Docker::default(),
            buildpack: BuildPackProject::default(),
            client_project: Project::default(),
        }
    }
    pub(crate) fn save(&self, state_path_dir: Option<&Path>) -> anyhow::Result<StateFiles> {
        StateFiles::save(self, state_path_dir)
    }

    pub(crate) fn load(runtime_dir: Option<&Path>) -> anyhow::Result<AutoPack> {
        StateFiles::new(runtime_dir)?.load_autopack()
    }

    pub(crate) fn validate(&self) -> anyhow::Result<bool> {
        debug!("validating autopack");

        self.docker.is_running()?;

        self.pack_cli.is_present()?;

        Ok(true)
    }

    pub(crate) fn load_validate(runtime_dir: Option<&Path>) -> Result<AutoPack, AppError> {
        let autopack = AutoPack::load(runtime_dir).map_err(|e| {
            crate::log::error("Please run `auto-pack init` to re-initialize autopack.");
            AppError::RunError("Failed loading autopack state", e)
        })?;

        autopack
            .validate()
            .map_err(|e| AppError::RunError("Failed validating autopack", e))?;

        Ok(autopack)
    }

    pub(crate) fn build(&self, clear_cache: bool) -> anyhow::Result<()> {
        trying("Auto packing project");

        self.validate()?;

        self.pack_cli.build_image(
            self.runtime.project_toml().as_str(),
            &self.runtime.proc_file().proc_default_command(),
            &self.runtime.proc_file().proc_file_path(),
            &self.runtime.proc_file().container_bindings_path(),
            &self.client_project.image_name,
            clear_cache,
        )?;

        Ok(())
    }

    pub(crate) async fn run(self, port: usize) -> anyhow::Result<()> {
        banner("Running autopack(ed) project");

        self.docker
            .run(
                self.client_project.image_name,
                &self.client_project.path,
                port,
            )
            .await?;

        Ok(())
    }
}
