mod spec;
#[cfg(test)]
mod tests;

use super::log::{success, trying};
use crate::error::AppError;
use npm_package_json::{Package, RepositoryReference};
use std::{env, fs, path::Path, vec};
use toml::Value;

pub(crate) use self::spec::*;

fn node_version_from_engine(pkg_json: Package) -> Option<String> {
    pkg_json.engines.get("node").cloned()
}

fn node_version_from_env() -> Option<String> {
    env::var("NODE_VERSION").ok()
}

fn node_version_from_nvmrc() -> Option<String> {
    fs::read_to_string({
        let this = env::current_dir().map(|mut p| {
            p.push(".nvmrc");
            p
        });
        let default = Path::new(".nvmrc").to_path_buf();
        match this {
            Ok(t) => t,
            Err(_) => default,
        }
    })
    .ok()
}

impl Default for BuildPackProject {
    fn default() -> Self {
        BuildPackProject::node_cra_template()
    }
}

impl BuildPackProject {
    fn node_cra_template() -> Self {
        let build = Some(Build {
            env: Some(vec![
                Env {
                    name: Some("BP_DISABLE_SBOM".to_string()),
                    value: Some("true".to_string()),
                },
                // Env {
                //     name: Some("NODE_ENV".to_string()),
                //     value: Some("development".to_string()),
                // },
                Env {
                    name: Some("BP_NODE_RUN_SCRIPTS".into()),
                    value: Some("build".into()),
                },
            ]),
            file_list: Some(FileList::Exclude(
                vec![
                    ".devcontainer",
                    ".husky",
                    ".editorconfig",
                    ".gitignore",
                    ".prettierrc",
                    "LICENSE.md",
                    "project-logo.png",
                    "README.md",
                    "node_modules",
                    "build",
                    "dist",
                ]
                .iter()
                .map(|&x| x.to_string())
                .collect(),
            )),
            buildpacks: Some(vec![BuildPack {
                id: None,
                buildpack_field: BuildPackField::Uri(Some("paketo-buildpacks/nodejs".to_string())),
            }]),
        });

        BuildPackProject {
            project: None,
            build,
            metadata: Some(Metadata::Meta(
                vec![(
                    "source".to_string(),
                    Value::String("auto-pack-0.0.1".to_string()),
                )]
                .into_iter()
                .collect(),
            )),
        }
    }

    pub(crate) fn enable_live_reload(&mut self) -> &mut Self {
        self.build.as_mut().map(|b| {
            b.env.as_mut().map(|e| {
                e.push(Env {
                    name: Some("BP_LIVE_RELOAD_ENABLED".to_string()),
                    value: Some("true".to_string()), // TODO: configure it
                })
            })
        });
        self
    }

    pub(crate) fn setup(pkg_json: &Package, live_reload: bool) -> Self {
        trying("Configuring Cloud Native Buildpack configuration");
        let mut base = BuildPackProject::node_cra_template();

        if live_reload {
            base.enable_live_reload();
        }

        let pkg_json_cloned = pkg_json.clone();
        base.project = Some(Project {
            id: Some(pkg_json_cloned.name),
            name: pkg_json_cloned.description,
            version: Some(pkg_json_cloned.version),
            authors: None,
            documentation_url: None,
            source_url: pkg_json_cloned.repository.map(|r| match r {
                RepositoryReference::Short(s) => s,
                RepositoryReference::Full(f) => f.url,
            }),
            licenses: pkg_json_cloned.license.map(|s| {
                vec![License {
                    licence_type: Some(s),
                    uri: None,
                }]
            }),
        });

        match base.build.as_mut() {
            Some(b) => match b.env.as_mut() {
                Some(e) => e.push(Env {
                    name: Some("BP_NODE_VERSION".to_string()),
                    value: node_version_from_env()
                        .or_else(node_version_from_nvmrc)
                        .or_else(|| node_version_from_engine(pkg_json.clone()))
                        .or_else(|| Some("^16.0.0".to_string())),
                }),
                None => {}
            },
            None => {}
        }

        success("Finished configuring Cloud Native Buildpacks");
        base
    }

    pub(crate) fn export_toml(&self, export_path: &Path) -> Result<(), AppError> {
        toml::to_string_pretty(self)
            .and_then(|s| {
                fs::write(export_path, s).map_err(|e| toml::ser::Error::Custom(e.to_string()))?;
                Ok(())
            })
            .map_err(|e| {
                AppError::PostConfigureError(
                    "Failed to export Buildpack project toml",
                    anyhow::anyhow!(e),
                )
            })
    }
}
