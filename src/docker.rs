use crate::{
    error::AppError,
    log::{command_err, command_out, error, instruct, success, trying},
};
use futures_util::TryFutureExt;
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    process::{Command, Stdio},
};
use tokio::io::AsyncBufReadExt;

use tracing::{debug, error};

#[cfg(target_os = "macos")]
fn docker_install_path() -> &'static str {
    "https://docs.docker.com/desktop/install/mac-install/"
}

#[cfg(target_os = "linux")]
fn docker_install_path() -> &'static str {
    "https://docs.docker.com/desktop/install/linux-install/"
}

#[cfg(target_os = "windows")]
fn docker_install_path() -> &'static str {
    "https://docs.docker.com/desktop/install/windows-install/"
}

fn ask_install_docker() -> Result<(), AppError> {
    debug!("asking user to open docker install page");
    instruct("Please install Docker and run init again.");
    let do_open = dialoguer::Confirm::new()
        .with_prompt("Press y or enter to open the webpage to install Docker.")
        .default(true)
        .interact()
        .map_err(|e| {
            error!(
                message = "Error in ask to install docker prompt",
                error = format!("{}", e)
            );
            AppError::IOError("Failed to launch the docker install page", e)
        })?;
    if do_open {
        debug!(message = "User gave consent to open docker install webpage");
        open::that(docker_install_path())
            .map(|_| {
                debug!(message = "Opened webpage", path = docker_install_path());
            })
            .map_err(|e| {
                error!(
                    message = "Error opening docker install path",
                    path = docker_install_path(),
                    error = format!("{}", e)
                );
                e
            })
            .unwrap();
    } else {
        debug!(message = "User did not gave consent to open docker install webpage");
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub(crate) struct Docker {
    version: String,
}

impl Docker {
    pub(crate) fn build(version: &str) -> Docker {
        Docker {
            version: version.to_string(),
        }
    }

    // pub(crate) fn version(&self) -> String {
    //     self.version.clone()
    // }

    pub(crate) fn check() -> Result<Docker, AppError> {
        debug!("Locating docker...");
        trying("Locating docker");

        Command::new("docker")
            .arg("--version")
            .output()
            .map_err(|e| anyhow::anyhow!(e))
            .and_then(|x| {
                if x.status.success() {
                    let docker = std::str::from_utf8(&x.stdout)
                        .map(Docker::build)
                        .map_err(|e| anyhow::anyhow!(e))?;
                    debug!("{} is installed", docker.version.trim_end());
                    success("Docker is installed");
                    Ok(docker)
                } else {
                    error!(
                        message = "Docker not found",
                        error = format!("{}", x.status)
                    );
                    error("Docker not found");
                    anyhow::bail!("Docker not found")
                }
            })
            .or_else(|_| {
                ask_install_docker()?;
                Err(AppError::PreconfigureError(
                    "Docker not found",
                    anyhow::anyhow!(""),
                ))
            })
    }

    pub(crate) fn is_running(&self) -> anyhow::Result<bool> {
        debug!("checking if docker is running");
        let cmd = Command::new("docker").arg("ps").arg("--quiet").output()?;
        cmd.status
            .success()
            .then(|| {
                debug!("docker is running");
                true
            })
            .ok_or_else(|| {
                error(
                    "Docker daemon is not running. Please start docker and run autopack build again.",
                );
                anyhow::anyhow!("Docker is not running")
            })
    }

    pub(crate) async fn stop_container(&self, image_name: &str) -> anyhow::Result<()> {
        let mut search_container_cmd = tokio::process::Command::new("docker");

        search_container_cmd
            .arg("ps")
            .args(["--filter", &format!(r#"name={}"#, image_name)])
            .args(["--filter", &format!(r#"status={}"#, "running")])
            .args(["--format", r#"{{.ID}}"#]);

        let search_container_child = search_container_cmd
            .stdout(Stdio::piped())
            .stdin(Stdio::null())
            .output()
            .await
            .map_err(|e| {
                anyhow::anyhow!("Failed creating docker container search command :: {}", e)
            })?;

        if search_container_child.status.success() {
            let container_id = std::str::from_utf8(&search_container_child.stdout)
                .map_err(|e| anyhow::anyhow!("Failed converting docker id to str :: {:?}", e))?
                .trim();

            if !container_id.is_empty() {
                debug!("trying to stop the container {}", container_id);

                let mut container_stop_cmd = tokio::process::Command::new("docker");
                container_stop_cmd
                    .args(["container", "stop", container_id])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .stdin(Stdio::null());

                loop {
                    let container_stop_child = container_stop_cmd
                        .output()
                        .and_then(|out| async move {
                            if out.status.success() {
                                Ok(out.stdout)
                            } else {
                                debug!(
                                    "error stopping, retrying... :: {}",
                                    std::str::from_utf8(&out.stderr).expect("could not get stderr")
                                );
                                Err(std::io::Error::new(std::io::ErrorKind::Other, ""))
                            }
                        })
                        .await;

                    if let Ok(out) = container_stop_child {
                        debug!(
                            "docker container {} stopped",
                            std::str::from_utf8(&out).expect("could not get stdout")
                        );
                        break;
                    }
                }
            } else {
                debug!("Docker container having name {} is not found", image_name);
            }
        }

        Ok(())
    }

    pub(crate) async fn run(
        self,
        image_name: String,
        project_dir: &Path,
        port: usize,
    ) -> anyhow::Result<()> {
        if let Err(err) = self.stop_container(&image_name).await {
            debug!("Failed stopping docker container :: {:?}", err)
        }

        let src_dir = project_dir
            .canonicalize()
            .or_else(|_| std::env::current_dir())
            .map_err(|e| anyhow::anyhow!("Failed getting the project dir :: {:?}", e))?;

        let mut docker_run_cmd = tokio::process::Command::new("docker");

        docker_run_cmd
            .arg("run")
            .arg("--interactive")
            .arg("--init")
            .arg("--rm")
            .args([
                "--mount",
                &format!(
                    r"type=bind,source={:#}/src,target=/workspace/src",
                    src_dir.display()
                ),
            ])
            .args(["-p", &format!("{}:8080", port)])
            .args(["--name", &image_name])
            .arg(image_name.clone());

        debug!("docker command :: {:?}", docker_run_cmd);

        let mut child = docker_run_cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed spawning docker run command :: {:?}", e))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("child does not have handle to stdout"))?;

        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow::anyhow!("child does not have handle to stderr"))?;

        let mut out_reader = tokio::io::BufReader::new(stdout).lines();
        let mut err_reader = tokio::io::BufReader::new(stderr).lines();

        #[cfg(not(target_os = "windows"))]
        let mut ctrlc_event =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?;

        #[cfg(target_os = "windows")]
        let mut ctrlc_event = tokio::signal::windows::ctrl_c()?;

        let handle = tokio::spawn(async move {
            tokio::select! {
                st = child.wait() => {
                    debug!("Failed running docker container");
                    if st.is_err() {
                        anyhow::bail!(st.unwrap_err())
                    }
                    Ok::<(), anyhow::Error>(())
                }
                ,
                _ = ctrlc_event.recv() => {
                    if let Err(err) = self.stop_container(&image_name).await {
                        debug!("Failed stopping docker container :: {:?}", err);
                    }

                    child.kill().map_err(|e| anyhow::anyhow!(e)).await?;

                    Ok::<(),anyhow::Error>(())
                }
            }
        });

        loop {
            tokio::select! {
                Ok(Some(line)) = out_reader.next_line() => {
                    command_out(&line);
                }
                Ok(Some(line)) = err_reader.next_line() => {
                    command_err(&line);
                }
                else => {
                    break;
                }
            }
        }

        handle
            .map_err(|e| anyhow::anyhow!("Error in the spawned task :: {:?}", e))
            .await??;

        Ok(())
    }
}
