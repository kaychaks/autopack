use super::Pack;
use crate::log::error;
use crate::log::{success, trying};
use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};
use tracing::{debug, error};

impl Pack {
    pub(crate) fn build_image(
        &self,
        project_toml: &str,
        start_cmd: &str,
        proc_file_path: &str,
        container_bindings_path: &str,
        image_name: &str,
        clear_cache: bool,
    ) -> anyhow::Result<()> {
        let mut cmd = Command::new(self.bin_file_path.clone());
        let cmd = cmd
            .arg("build")
            .args(["-d", project_toml])
            .args(["-D", start_cmd])
            .arg("--volume")
            .arg(&format!(
                r#"{}:{}"#,
                proc_file_path, container_bindings_path
            ))
            .arg(image_name);

        if clear_cache {
            cmd.arg("--clear-cache");
        }

        debug!("Executing {:?}", cmd);

        let mut child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed running pack cli :: {:?}", e))?;
        let stdout = child
            .stdout
            .take()
            .ok_or("Cound not capture stdout")
            .unwrap();
        let stderr = child
            .stderr
            .take()
            .ok_or("Cound not capture stderr")
            .unwrap();

        let out_reader = BufReader::new(stdout);
        let err_reader = BufReader::new(stderr);

        let mut p_install_start_marker = false;

        out_reader.lines().filter_map(|l| l.ok()).for_each(|l| {
            let container_installing = l.contains("Pulling from paketobuildpacks/builder");
            let container_installed =
                l.contains("Status: Image is up to date for paketobuildpacks/run:base-cnb");

            let packages_installing = l.contains("Executing build environment install process");
            let completed = l.contains("Completed in");
            let image_built = l.contains("Successfully built image");

            if container_installing {
                trying("Downloading docker images");
            }
            if container_installed {
                success("Docker images downloaded");
                trying("Setting up buildpacks");
            }

            if packages_installing {
                success("Buildpacks setup done");
                trying("Installing NPM packages");
                p_install_start_marker = true;
            }

            if completed && p_install_start_marker {
                p_install_start_marker = false;
                success("Installed NPM packages");
                trying("Finalizing image");
            }

            if image_built {
                success(&format!("Image {} built successfully", image_name));
            }

            debug!("{}", l);
        });
        err_reader.lines().for_each(|l| error!("{:?}", l));

        let out = child.wait_with_output()?;
        if out.status.success() {
            debug!("Auto packing project complete");
            success("Auto packing project complete");
        } else {
            error!("Auto packing project failed");
            error("Auto packing project complete");
            anyhow::bail!("Failed auto packing project")
        }

        Ok(())
    }
}
