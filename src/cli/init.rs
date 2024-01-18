use crate::{
    autopack::AutoPack,
    buildpack::BuildPackProject,
    docker::Docker,
    error::AppError,
    pack::Pack,
    package_json::{CreatePackageJson, Project},
    runtime::Runtime,
};
use std::{env, io, path::PathBuf};
use tracing::{debug, warn};

/// Builder for [`crate::app::autopack::AutoPack`]
pub struct Init {
    package_json: Project,
    docker: Docker,
    buildpack: Option<BuildPackProject>,
    runtime: Option<Runtime>,
    pack_cli: Option<Pack>,
}

impl Init {
    pub(crate) fn pre_configure(client_project_path: Option<PathBuf>) -> Result<Self, AppError> {
        debug!("Begin pre-configure");
        let path = client_project_path
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
            .or_else(|_| env::current_dir())
            .map_err(|e| {
                let msg = "No client project path given, falling back on current directory";
                warn!(msg);
                AppError::PreconfigureError(msg, anyhow::anyhow!(e))
            })?;

        let docker = Docker::check()?;
        let package_json = CreatePackageJson::new(&path).build()?;

        debug!("End pre-configure");
        Ok(Init {
            package_json,
            docker,
            buildpack: None,
            runtime: None,
            pack_cli: None,
        })
    }

    pub(crate) fn configure(&mut self, live_reload: bool) -> Result<&mut Self, AppError> {
        debug!("Begin configure");
        let buildpack = BuildPackProject::setup(&self.package_json.package_json, live_reload);
        self.buildpack = Some(buildpack);

        debug!("End configure");
        Ok(self)
    }

    pub(crate) async fn post_configure(
        &mut self,
        force_create_runtime: bool,
        live_reload: bool
    ) -> Result<&mut Self, AppError> {
        debug!("Begin post-configure");
        // create runtime
        let runtime = Runtime::builder(self.package_json.path.clone().as_path())
            .dir(force_create_runtime)?
            .proc_file(live_reload)?
            .build();

        let filename = "project.toml";
        let mut path = runtime.dir().clone();
        path.push(filename);

        // export BP to toml
        self.buildpack
            .as_ref()
            .ok_or_else(|| {
                AppError::PostConfigureError(
                    "Buildpack project not available to export",
                    anyhow::anyhow!(""),
                )
            })
            .and_then(|bp| bp.export_toml(&path))?;

        // Install pack cli
        let mut pack = Pack::builder(&runtime.dir()).build();
        let pack = pack.install().await?;

        self.runtime = Some(runtime);
        self.pack_cli = Some(pack.to_owned());
        debug!("End post-configure");
        Ok(self)
    }

    pub(crate) fn install(&mut self) -> AutoPack {
        AutoPack {
            runtime: self.runtime.clone().unwrap_or_default(),
            pack_cli: self.pack_cli.clone().unwrap_or_default(),
            buildpack: self.buildpack.clone().unwrap_or_default(),
            docker: self.docker.clone(),
            client_project: self.package_json.clone(),
        }
    }
}
