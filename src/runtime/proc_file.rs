use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

/// Procfile is required to override the default start command in the container.
///
/// [Paketo's Procfile Buildpack](https://github.com/paketo-buildpacks/procfile) supports Procfiles which are either made available via a normal file or via a file organization following the [K8S Service Binding Spec](https://github.com/servicebinding/spec).
///
/// We are following the Service Binding approach so that there are minimum changes incurred in the application folder structure.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub(crate) struct ProcFile {
    command_entries: HashMap<String, String>,
    default_command: String,
    binding_provider: String,
    binding_type: String,
    file_path: Option<PathBuf>,
}

impl Default for ProcFile {
    fn default() -> Self {
        ProcFile {
            command_entries: HashMap::new(),
            binding_provider: "autopack".to_string(),
            binding_type: "Procfile".to_string(),
            default_command: "override-start".to_string(),
            file_path: Some(Path::new(".autopack").to_path_buf().join("Procfile")),
        }
    }
}

impl ProcFile {
    pub(crate) fn builder() -> ProcFileBuilder {
        let pf = ProcFile::default();
        ProcFileBuilder {
            command_entries: pf.command_entries,
            default_command: pf.default_command,
            binding_provider: pf.binding_provider,
            binding_type: pf.binding_type,
            file_path: pf.file_path,
        }
    }

    pub(crate) fn proc_file_path(&self) -> String {
        self.file_path
            .clone()
            .unwrap_or_else(|| ProcFile::default().file_path.unwrap())
            .to_str()
            .expect("expected a valid procfile folder")
            .to_string()
    }

    pub(crate) fn proc_default_command(&self) -> String {
        self.default_command.clone()
    }

    pub(crate) fn container_bindings_path(&self) -> String {
        "/platform/bindings/Procfile".to_string()
    }
}

pub(crate) struct ProcFileBuilder {
    command_entries: HashMap<String, String>,
    default_command: String,
    binding_provider: String,
    binding_type: String,
    file_path: Option<PathBuf>,
}

impl ProcFileBuilder {
    pub(crate) fn command_entry(
        &mut self,
        command_type: &str,
        command: &str,
    ) -> &mut ProcFileBuilder {
        self.command_entries
            .insert(command_type.to_string(), command.to_string());
        self
    }

    pub(crate) fn override_start_entry(&mut self, live_reload: bool) -> &mut ProcFileBuilder {
        let mut watch_cmd = None;
        let serve_cmd = "serve -s build -l 8080".to_string();
        if live_reload {
            let build_cmd = "npm run build".to_string();
            watch_cmd = Some(format!(
                r#"watchexec --restart --shell none --watch /workspace/src -- bash -c "{} && {}""#,
                build_cmd,
                serve_cmd
            ));
        }

        self.command_entry(
            "override-start",
            &format!(
                r#"npm install -g serve && {}"#,
                watch_cmd.unwrap_or(serve_cmd)
            ),
        );
        self
    }

    pub(crate) fn export(&mut self, export_path: &Path) -> anyhow::Result<&mut ProcFileBuilder> {
        let dir = export_path.to_path_buf().join("Procfile");

        if !dir.exists() {
            fs::create_dir(dir.clone())?;
        }

        fs::write(dir.join("type"), self.binding_type.clone())?;
        fs::write(dir.join("provider"), self.binding_provider.clone())?;
        fs::write(
            dir.join("Procfile"),
            self.command_entries
                .clone()
                .into_iter()
                .fold("".to_string(), |acc, x| {
                    format!("{}: {}\n{}", x.0, x.1, acc)
                }),
        )?;

        self.file_path = Some(dir.canonicalize().map_err(|e| anyhow::anyhow!(e))?);
        Ok(self)
    }

    pub(crate) fn build(&mut self) -> ProcFile {
        ProcFile {
            command_entries: self.command_entries.clone(),
            default_command: self.default_command.clone(),
            binding_provider: self.binding_provider.clone(),
            binding_type: self.binding_type.clone(),
            file_path: self.file_path.clone(),
        }
    }
}
