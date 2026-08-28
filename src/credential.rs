use anyhow::{Context, Result, bail};
use flate2::read::ZlibDecoder;
use serde::Deserialize;
use std::sync::OnceLock;
use std::{fs::File, io::Read, path::Path};

#[derive(Debug, Deserialize)]
pub struct Credential {
    pub server_id: String,
    pub endpoint: String,
    pub password: String,
}

static CREDENTIAL: OnceLock<Credential> = OnceLock::new();

pub fn load(path: impl AsRef<Path>) -> Result<()> {
    if CREDENTIAL.get().is_some() {
        bail!("Server credential has already been loaded");
    }

    let path = path.as_ref();
    let file = File::open(path)
        .with_context(|| format!("failed to open credential file {}", path.display()))?;
    let mut decoder = ZlibDecoder::new(file);
    let mut json = Vec::new();
    decoder
        .read_to_end(&mut json)
        .with_context(|| format!("failed to decompress credential file {}", path.display()))?;

    let credential: Credential = serde_json::from_slice(&json)
        .with_context(|| format!("invalid credential JSON in {}", path.display()))?;
    validate(&credential)?;
    CREDENTIAL
        .set(credential)
        .map_err(|_| anyhow::anyhow!("Server credential has already been loaded"))?;
    Ok(())
}

pub fn get() -> Result<&'static Credential> {
    CREDENTIAL
        .get()
        .context("Server credential is not initialized; call credential::load first")
}

fn validate(credential: &Credential) -> Result<()> {
    let valid_id = credential.server_id.len() == 18
        && credential.server_id.starts_with("0x")
        && credential.server_id[2..]
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit());
    if !valid_id {
        bail!("server credential validation failed");
    }
    if credential.endpoint.trim().is_empty() {
        bail!("server credential validation failed");
    }
    if credential.password.is_empty() {
        bail!("server credential validation failed");
    }
    Ok(())
}
