use aes_gcm::aead::consts::{U12, U32};
use aes_gcm::aead::generic_array::GenericArray;
use aes_gcm::{aead::Aead, KeyInit};
use blake2::{Blake2b512, Digest};
use rand::Rng;
use std::fs;
use tracing::debug;

use crate::autopack::filesystem::StateFiles;

pub(super) struct APCrypto {
    cipher_key: GenericArray<u8, U32>,
    nonce: GenericArray<u8, U12>,
    content_hash: Option<Vec<u8>>,
}

pub(super) struct APEncryptVal {
    pub(super) cipher_text: Vec<u8>,
    pub(super) nonce: GenericArray<u8, U12>,
}

impl APEncryptVal {
    pub(super) fn new(ct: &str, n: &[u8; 12]) -> Self {
        APEncryptVal {
            cipher_text: hex::decode(ct).unwrap(),
            nonce: *GenericArray::from_slice(n),
        }
    }
}

pub(super) struct APCryptoBuilder {
    cipher_key: GenericArray<u8, U32>,
    // nonce_key: Option<GenericArray<u8, U12>>,
    content_hash: Option<Vec<u8>>,
}

impl APCrypto {
    pub(super) fn builder() -> APCryptoBuilder {
        let cipher_key =
            &hex::decode("b52c505a37d78eda5dd34f20c22540ea1b58963cf8e5bf8ffa85f9f2492505b4")
                .unwrap();
        let cipher_key = *GenericArray::from_slice(cipher_key);

        APCryptoBuilder {
            cipher_key,
            // nonce_key: None,
            content_hash: None,
        }
    }

    pub(super) fn encrypt(&self) -> anyhow::Result<APEncryptVal> {
        let content_hash = self
            .content_hash
            .clone()
            .ok_or_else(|| anyhow::anyhow!("no content to encrypt"))?;

        let cipher = aes_gcm::Aes256Gcm::new(&self.cipher_key);

        let cipher_text = cipher
            .encrypt(&self.nonce, content_hash.as_slice())
            .map_err(|e| anyhow::anyhow!("encryption error :: {:?}", e))?;

        Ok(APEncryptVal {
            cipher_text,
            nonce: self.nonce,
        })
    }

    pub(super) fn validate_hashes(state_files: &StateFiles) -> anyhow::Result<&StateFiles> {
        debug!("Validating content hashes");

        let StateFiles {
            state_content,
            checksum,
            ..
        } = state_files;

        let reference_crypto_val = {
            let check_content = fs::read_to_string(checksum.clone())?;
            let lines: Vec<_> = check_content.lines().map(|l| l.trim()).collect();
            let nonce = hex::decode(lines[1]).unwrap();
            let mut x = [0; 12];
            x[..=11].clone_from_slice(&nonce);

            APEncryptVal::new(lines[0], &x)
        };

        let decrypted = APCrypto::builder().decrypt(reference_crypto_val)?;

        let existing = {
            let state_content = fs::read(state_content.clone())?;
            APCrypto::builder().content_hash(state_content).build()
        };

        if decrypted.content_hash == existing.content_hash {
            Ok(state_files)
        } else {
            anyhow::bail!("Hashes don't match")
        }
    }
}

impl APCryptoBuilder {
    pub(super) fn content_hash(&mut self, contents: Vec<u8>) -> &mut Self {
        self.content_hash = Some(
            Blake2b512::new_with_prefix(contents)
                .finalize()
                .as_slice()
                .into(),
        );
        self
    }

    pub(super) fn build(&self) -> APCrypto {
        let rand_gen = rand::thread_rng().gen::<[u8; 12]>();

        let nonce = *GenericArray::from_slice(&rand_gen);

        APCrypto {
            cipher_key: self.cipher_key,
            content_hash: self.content_hash.clone(),
            nonce,
        }
    }

    pub(super) fn decrypt(&mut self, val: APEncryptVal) -> anyhow::Result<APCrypto> {
        let cipher = aes_gcm::Aes256Gcm::new(&self.cipher_key);

        let content_hash = cipher
            .decrypt(&val.nonce, val.cipher_text.as_slice())
            .map_err(|e| anyhow::anyhow!("Decryption error :: {:?}", e.to_string()))?;

        self.content_hash = Some(content_hash);

        Ok(APCrypto {
            cipher_key: self.cipher_key,
            content_hash: self.content_hash.clone(),
            nonce: val.nonce,
        })
    }
}
