use anyhow::{anyhow, Context as _, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD as base64engine, Engine as _};
use orion::aead::{self, SecretKey};
use std::{
    env,
    fs::{self, File},
    io::{BufRead as _, BufReader},
};

pub struct WorkgroupKey(SecretKey);

impl WorkgroupKey {
    pub fn load() -> Result<Self> {
        Ok(WorkgroupKey(SecretKey::from_slice(
            &base64engine.decode(
                &BufReader::new(File::open(format!(
                    "{}/.ssh/workgroup",
                    env::var("HOME").unwrap_or_default()
                ))?)
                .lines()
                .next()
                .context("Workgroup key file is corrupted")??,
            )?,
        )?))
    }

    pub fn create() -> Result<()> {
        Ok(fs::write(
            format!("{}/.ssh/workgroup", env::var("HOME").unwrap_or_default()),
            base64engine.encode(SecretKey::default().unprotected_as_bytes()),
        )?)
    }
}

pub struct SshChain(pub Vec<String>);

impl SshChain {
    fn open_impl(key: &WorkgroupKey) -> Result<Vec<String>> {
        Ok(String::from_utf8(aead::open(
            &key.0,
            &base64engine.decode(env::var("WORKGROUP_CHAIN")?)?,
        )?)?
        .split_whitespace()
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>())
    }

    pub fn open(key: Option<&WorkgroupKey>) -> SshChain {
        let ssh_chain = key
            .context("No workgroup key passed")
            .and_then(Self::open_impl)
            .and_then(|chain| {
                if chain.is_empty() {
                    Err(anyhow!("Empty ssh chain, but decoded"))
                } else {
                    Ok(chain)
                }
            });

        SshChain(match (ssh_chain, env::var("SSH_CONNECTION")) {
            (Err(_), Err(_)) => vec![],
            (Err(_), Ok(conn)) => vec![conn.split_whitespace().next().unwrap_or("?").to_owned()],
            (Ok(ch), _) => ch,
        })
    }

    fn seal_impl(&self, key: &WorkgroupKey) -> Result<String> {
        Ok(base64engine.encode(aead::seal(&key.0, self.0.join(" ").as_bytes())?))
    }

    #[must_use]
    pub fn seal(&self, key: &WorkgroupKey) -> String {
        self.seal_impl(key).unwrap_or_default()
    }
}
