mod image;

use super::log::{success, trying};
use crate::error::AppError;
use anyhow::anyhow;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};
use tracing::debug;

#[cfg(target_os = "macos")]
fn release_url(pack_version: &str) -> String {
    format!(
        "https://github.com/buildpacks/pack/releases/download/v{}/pack-v{}-macos.tgz",
        pack_version, pack_version
    )
}

#[cfg(all(target_os = "linux", target_arch = "arm"))]
fn release_url(pack_version: &str) -> String {
    format!(
        "https://github.com/buildpacks/pack/releases/download/v{}/pack-v{}-linux-arm64.tgz",
        pack_version, pack_version
    )
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn release_url(pack_version: &str) -> String {
    format!(
        "https://github.com/buildpacks/pack/releases/download/v{}/pack-v{}-linux.tgz",
        pack_version, pack_version
    )
}

#[cfg(target_os = "windows")]
fn release_url(pack_version: &str) -> String {
    format!(
        "https://github.com/buildpacks/pack/releases/download/v{}/pack-v{}-windows.zip",
        pack_version, pack_version
    )
}

/// Internal binary to be used to create the CNBs
/// https://buildpacks.io/docs/tools/pack/
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct Pack {
    /// directory within `.autopack` holding the binary. default to `.bin`
    pub(crate) bin_dir_path: PathBuf,
    /// OS specific full file path to the pack cli binary. default to `.autopack/.bin/pack` in unix and `.autopack/.bin/pack.exe` in windows
    pub(crate) bin_file_path: PathBuf,
    /// OS specific download URL for pack cli
    pub(crate) release_url: String,
    /// pack cli version
    pub(crate) release_version: String,
}

impl Default for Pack {
    fn default() -> Self {
        Pack {
            bin_dir_path: PathBuf::new().join(".autopack").join(".bin"),
            bin_file_path: PathBuf::new().join(".autopack").join(".bin").join("pack"),
            release_url: "".to_string(),
            release_version: "".into(),
        }
    }
}

impl Pack {
    pub(crate) fn builder(runtime_dir: &Path) -> PackBuilder {
        PackBuilder::new(runtime_dir)
    }

    pub(crate) fn is_present(&self) -> anyhow::Result<bool> {
        debug!("Checking if pack-cli is present");

        self.bin_file_path
            .exists()
            .then(|| true)
            .ok_or_else(|| anyhow::anyhow!("Pack cli not available"))
    }

    pub(crate) async fn install(&mut self) -> Result<&Self, AppError> {
        trying("Installing pack cli");
        let x = self.is_present().unwrap_or(false);
        if x {
            debug!("Pack cli present, skipping download and install");
            success("Pack cli already present");
            Ok(self)
        } else {
            // download release binary
            self.download_and_extract().await.map_err(|e| {
                AppError::PostConfigureError("Failed to download and extract Pack cli", anyhow!(e))
            })?;

            // check if the installation is fine
            let version = self.cli_version().map_err(|e| {
                AppError::PostConfigureError("Failed to get the pack cli version", e)
            })?;

            debug!("Installed pack cli version {}", version);
            success("Installed pack cli");
            Ok(self)
        }
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    fn decompress<P: AsRef<Path>>(&self, path: P, dest: P) -> anyhow::Result<File> {
        use flate2::read::GzDecoder;
        use tar::Archive;

        let tgz = File::open(path)?;
        let tar = GzDecoder::new(tgz);
        let mut archive = Archive::new(tar);
        archive.unpack(dest.as_ref())?;
        File::open(&self.bin_file_path).map_err(|e| anyhow!("Error during decompress. Error {}", e))
    }

    #[cfg(target_os = "windows")]
    fn decompress<P: AsRef<Path>>(&self, path: P, dest: P) -> anyhow::Result<File> {
        let mut archive = zip::ZipArchive::new(File::open(path)?)?;
        archive.extract(dest.as_ref())?;
        File::open(&self.bin_file_path).map_err(|e| anyhow!("Error during decompress. Error {}", e))
    }

    async fn download_and_extract(&mut self) -> anyhow::Result<&Self> {
        std::fs::create_dir(&self.bin_dir_path)?;

        let mut temp_file = tempfile::Builder::new().tempfile_in(&self.bin_dir_path)?;

        let client = reqwest::Client::new();
        let response = client.get(self.release_url.as_str()).send().await?;
        let bin_size = response
            .content_length()
            .ok_or_else(|| anyhow::Error::msg("Error getting content length"))?;

        let mut start = 0_u64;
        let end = bin_size - 1;

        let bar = ProgressBar::new(end);
        bar.set_style(
            ProgressStyle::default_bar()
                .template(
                    "[{elapsed_precise}] |{bar:40.cyan/blue}| {bytes:>7}/{total_bytes:7} {msg} [{eta}]",
                )
                .unwrap()
                .progress_chars("█▛▌▖  "),
        );
        bar.set_message("Downloading pack cli");

        let mut bin_stream = response.bytes_stream();
        while let Some(bs) = bin_stream.next().await {
            let chunk = bs?;
            let chunk_size: u64 = chunk.len() as u64;
            temp_file.write(&chunk).map_err(|e| {
                anyhow::Error::msg(format!("Error while writing content to file. Error {}", e))
            })?;
            let progress = std::cmp::min(chunk_size, end - start + 1);
            start += progress;
            bar.inc(progress);
        }

        bar.finish_with_message("Downloaded pack cli");
        bar.finish_and_clear();

        let f = self.decompress(temp_file.path(), &self.bin_dir_path)?;
        set_permissions(f)?;
        Ok(self)
    }

    pub(crate) fn cli_version(&self) -> anyhow::Result<String> {
        let mut cmd = std::process::Command::new(&self.bin_file_path);
        let out = cmd.arg("--version").output()?;
        if out.status.success() {
            std::str::from_utf8(&out.stdout)
                .map(|x| x.to_string())
                .map_err(|e| anyhow::anyhow!(e))
        } else {
            anyhow::bail!("Error getting pack output")
        }
    }
}
pub(crate) struct PackBuilder {
    /// directory within `.autopack` holding the binary. default to `.bin`
    pub(crate) bin_dir_path: PathBuf,
    /// OS specific full file path to the pack cli binary. default to `.autopack/.bin/pack` in unix and `.autopack/.bin/pack.exe` in windows
    pub(crate) bin_file_path: PathBuf,
    /// OS specific download URL for pack cli
    pub(crate) release_url: String,
    /// pack cli version
    pub(crate) release_version: String,
}

impl PackBuilder {
    pub(crate) fn new(runtime_dir: &Path) -> Self {
        let release_version = "0.27.0".to_string();
        let bin_dir_path = PathBuf::new().join(runtime_dir).join(".bin");
        PackBuilder {
            bin_dir_path: bin_dir_path.clone(),
            release_version: release_version.clone(),
            release_url: release_url(&release_version),

            #[cfg(any(target_os = "linux", target_os = "macos"))]
            bin_file_path: bin_dir_path.join("pack"),

            #[cfg(target_os = "windows")]
            bin_file_path: bin_dir_path.join("pack.exe"),
        }
    }

    pub(crate) fn build(&mut self) -> Pack {
        Pack {
            bin_dir_path: self.bin_dir_path.clone(),
            bin_file_path: self.bin_file_path.clone(),
            release_url: self.release_url.clone(),
            release_version: self.release_version.clone(),
        }
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn set_permissions(f: File) -> anyhow::Result<File> {
    use std::os::unix::prelude::PermissionsExt;

    let mut perms = f.metadata()?.permissions();
    perms.set_mode(0o700);
    f.set_permissions(perms)?;
    Ok(f)
}

#[cfg(target_os = "windows")]
fn set_permissions(f: File) -> anyhow::Result<File> {
    Ok(f)
}

#[cfg(test)]
mod tests {
    use std::env;

    use crate::runtime::Runtime;

    use super::*;

    #[tokio::test]
    async fn pack_version_test() {
        let rt = Runtime::builder(env::current_dir().unwrap().as_path())
            .dir(false)
            .unwrap()
            .build();
        let mut pack_builder = Pack::builder(rt.dir().as_path());
        let mut pack = pack_builder.build();
        let pack = pack.install().await.unwrap();

        assert!(pack.cli_version().unwrap().contains(&pack.release_version));
    }
}
