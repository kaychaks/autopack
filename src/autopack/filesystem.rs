use super::{
    crypto::{APCrypto, APEncryptVal},
    AutoPack,
};
use crate::{log::error, runtime::Runtime};
use rmp_serde::{decode, encode};
use std::{
    ffi::OsStr,
    fs::{self, File},
    path::{Path, PathBuf},
    time,
};

pub(crate) struct StateFiles {
    pub(crate) state_content: PathBuf,
    pub(crate) checksum: PathBuf,
    // pub(crate) state_dir: PathBuf,
}

impl StateFiles {
    pub(super) fn new(runtime_dir: Option<&Path>) -> anyhow::Result<StateFiles> {
        let runtime_dir = runtime_dir
            .map(|e| e.to_path_buf())
            .unwrap_or_else(|| Runtime::default().dir());

        let state_dir = runtime_dir.join(".state");

        let fs = state_dir
            .read_dir()
            .map_err(|e| {
                error("Failed loading autopack state");
                anyhow::anyhow!(
                    "State path {} does not exist :: {:?}",
                    state_dir.display(),
                    e.to_string()
                )
            })?
            .fold((None, None), |mut acc, f| match f {
                Ok(x) => {
                    let ext = x.path();
                    let ext = ext.extension().and_then(OsStr::to_str).unwrap_or_default();
                    if ext == "checksum" {
                        acc.1 = Some(x.path());
                    } else {
                        acc.0 = Some(x.path());
                    }
                    acc
                }
                _ => acc,
            });

        let state_content =
            fs.0.ok_or_else(|| anyhow::anyhow!("State content path not found"))?;
        let checksum =
            fs.1.ok_or_else(|| anyhow::anyhow!("State content checksum path not found"))?;

        Ok(StateFiles {
            state_content,
            checksum,
            // state_dir,
        })
    }
    pub(super) fn save(
        autopack: &AutoPack,
        save_path: Option<&Path>,
    ) -> anyhow::Result<StateFiles> {
        let default_path = autopack.runtime.dir().join(".state");
        let save_path = save_path.unwrap_or(&default_path);

        if !save_path.exists() {
            fs::create_dir(save_path)?;
        } else {
            fs::remove_dir_all(save_path)?;
            fs::create_dir(save_path)?;
        }

        let version = autopack.client_project.package_json.version.clone();

        let state_file_name = format!(
            "{}_{}_{}",
            autopack.client_project.package_json.name,
            version,
            time::SystemTime::now()
                .duration_since(time::UNIX_EPOCH)
                .map_err(|e| anyhow::anyhow!("could not get current time :: {:?}", e))?
                .as_millis()
        );
        let state_file = save_path.join(state_file_name.clone());

        let save_file = save_path.join(state_file);
        let checksum_file = save_path.join(format!("{}.checksum", state_file_name));
        let contents = encode::to_vec(&autopack)?;

        let a = APCrypto::builder().content_hash(contents.clone()).build();

        let APEncryptVal { cipher_text, nonce } = a.encrypt()?;

        fs::write(save_file.clone(), contents)
            .map_err(|e| anyhow::anyhow!("error writing state file :: {:?}", e))?;

        fs::write(
            checksum_file.clone(),
            format!(
                r"{}
{}",
                hex::encode(cipher_text),
                hex::encode(nonce)
            ),
        )
        .map_err(|e| anyhow::anyhow!("error writing checksum :: {:?}", e))?;

        Ok(StateFiles {
            checksum: checksum_file,
            state_content: save_file,
            // state_dir: save_path.to_path_buf(),
        })
    }
    pub(super) fn load_autopack(&self) -> anyhow::Result<AutoPack> {
        APCrypto::validate_hashes(self)?;

        let autpack = decode::from_read(File::open(self.state_content.clone()).map_err(|e| {
            anyhow::anyhow!("Failed to open file at {:?} :: {:?}", self.state_content, e)
        })?)
        .map_err(|e| {
            anyhow::anyhow!(
                "Fail to deserialize application state from file at {:?} :: {:?}",
                self.state_content,
                e
            )
        })?;
        Ok(autpack)
    }
}
