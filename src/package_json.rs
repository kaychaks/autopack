use crate::{
    error::AppError,
    log::{error, success, trying},
};
use npm_package_json::Package;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{error, info};

fn check_package_json(mod_path: &Path) -> Result<String, AppError> {
    trying("Locating package.json");
    mod_path
        .canonicalize()
        .map_err(|e| AppError::IOError("Failed to locate package.json", e))
        .and_then(|mut path| {
            path.push("package.json");
            let p = path
                .clone()
                .into_os_string()
                .into_string()
                .map_err(AppError::OSPathError);

            if path.exists() {
                info!(message = "Package JSON found", package_json_path = ?p);
                success("Package JSON found");
                p
            } else {
                error("Package JSON not found");
                error!(message = "Package JSON not found", package_json_path = ?p);
                Err(AppError::PackageJSONNotFound)
            }
        })
}

fn get_pkg_json(path: &str) -> Result<Package, AppError> {
    Package::from_path(path).map_err(|e| match e {
        npm_package_json::Error::Io(e) => AppError::IOError("Failed to parse package.json", e),
        npm_package_json::Error::Parse(_) => AppError::ReactScriptsNotFound,
    })
}

fn check_react_script(package_json_path: &str) -> Result<Package, AppError> {
    trying("Checking react-scripts within the project");
    let pkg = get_pkg_json(package_json_path)?;

    if pkg.dev_dependencies.contains_key("react-scripts")
        || pkg.dependencies.contains_key("react-scripts")
    {
        success("react-scripts found");
        Ok(pkg)
    } else {
        error("react-scripts not found");
        Err(AppError::ReactScriptsNotFound)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub(crate) struct Project {
    pub(crate) path: PathBuf,
    pub(crate) package_json: Package,
    pub(crate) image_name: String,
}

pub(crate) struct CreatePackageJson {
    path: PathBuf,
}

impl CreatePackageJson {
    pub(crate) fn new(p: &Path) -> CreatePackageJson {
        CreatePackageJson {
            path: p.to_path_buf(),
        }
    }

    pub(crate) fn build(&self) -> Result<Project, AppError> {
        check_package_json(&self.path)
            .map(|s| check_react_script(&s))?
            .map(|p| {
                let image_name = p.name.trim().replace(' ', "_");
                Project {
                    package_json: p,
                    path: self.path.clone(),
                    image_name,
                }
            })
    }
}
