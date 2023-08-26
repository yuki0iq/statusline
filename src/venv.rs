use crate::{Icon, Icons};
use anyhow::Result;
use std::{
    env,
    ffi::OsStr,
    fs::File,
    io::{BufRead, BufReader},
    ops::Not,
    path::{Path, PathBuf},
};

pub struct Venv {
    name: String,
    version: String,
}

impl Venv {
    pub fn get() -> Option<Venv> {
        let path = PathBuf::from(env::var("VIRTUAL_ENV").ok()?);
        let name = venv_name(&path).to_string();
        let version = venv_ver(&path)
            .unwrap_or_default()
            .unwrap_or("<sys?>".to_string());

        Some(Venv { name, version })
    }

    pub fn pretty(&self, icons: &Icons) -> String {
        format!("{} {}|{}", icons(Icon::Venv), self.version, self.name)
    }
}

fn venv_name(path: &Path) -> &str {
    path.ancestors()
        .filter_map(Path::file_name)
        .filter_map(OsStr::to_str)
        .find_map(|name| {
            ["venv", "env", "virtualenv"]
                .contains(&name)
                .not()
                .then_some(
                    ["venv", "virtualenv", "env", "-", "_"]
                        .iter()
                        .fold(name, |s, suf| s.strip_suffix(suf).unwrap_or(s)),
                )
        })
        .unwrap_or("<venv>")
}

fn venv_ver(path: &Path) -> Result<Option<String>> {
    Ok(BufReader::new(File::open(path.join("pyvenv.cfg"))?)
        .lines()
        .find_map(|line| {
            Some(
                line.ok()?
                    .strip_prefix("version")?
                    .trim_start_matches(' ')
                    .strip_prefix('=')?
                    .trim_start_matches(' ')
                    .to_string(),
            )
        }))
}
