use std::ffi::OsString;
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Preconfigure error :: {0} :: {:?}", .1)]
    PreconfigureError(&'static str, #[source] anyhow::Error),
    #[error("Package JSON not found")]
    PackageJSONNotFound,
    #[error("React scripts not found")]
    ReactScriptsNotFound,
    #[error("Some OS specific error :: {:?}", .0.to_str().unwrap_or(""))]
    OSPathError(OsString),
    #[error("Some IO error :: {0} :: {:?}", .1)]
    IOError(&'static str, #[source] std::io::Error),
    #[error("Postconfigure error :: {0} :: {:?}", .1)]
    PostConfigureError(&'static str, #[source] anyhow::Error),
    #[error("Build error :: {0} :: {:?}", .1)]
    BuildError(&'static str, #[source] anyhow::Error),
    #[error("Run error :: {0} :: {:?}", .1)]
    RunError(&'static str, #[source] anyhow::Error),
}

impl AppError {
    pub fn handle(&self) {
        match &self {
            AppError::IOError(context, e) => {
                error!("IO Error :: {} :: {:?}", context, e);
            }
            AppError::OSPathError(context) => {
                error!("OS Specific error :: {:?}", context);
            }
            AppError::PackageJSONNotFound => {
                error!("Package JSON not found");
            }
            AppError::PostConfigureError(context, e) => {
                error!("Post configure error :: {} :: {:?}", context, e);
            }
            AppError::PreconfigureError(context, e) => {
                error!("Pre configure error :: {} :: {:?}", context, e);
            }
            AppError::ReactScriptsNotFound => {
                error!("React scripts not found");
            }
            AppError::BuildError(context, e) => {
                error!("Build failure :: {} :: {:?}", context, e)
            }
            AppError::RunError(context, e) => {
                error!("Run failure :: {} :: {:?}", context, e)
            }
        };
    }
}
