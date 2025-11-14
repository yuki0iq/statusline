use crate::{Block, Environment, Icon, IconMode, Pretty, Style as _};
use anyhow::Result;
use std::{
    ffi::OsStr,
    fs::File,
    io::{BufRead as _, BufReader},
    path::{Path, PathBuf},
};

pub struct Venv {
    name: String,
    version: String,
}

super::register_block!(Venv);

impl Block for Venv {
    fn new(_: &Environment) -> Option<Box<dyn Block>> {
        let path = PathBuf::from(std::env::var("VIRTUAL_ENV").ok()?);
        let name = venv_name(&path).to_owned();
        let version = venv_ver(&path)
            .unwrap_or_default()
            .unwrap_or("<sys?>".to_owned());

        Some(Box::new(Venv { name, version }))
    }
}

impl Pretty for Venv {
    fn pretty(&self, mode: IconMode) -> Option<String> {
        Some(
            format!("[{} {}|{}]", self.icon(mode), self.version, self.name)
                .visible()
                .yellow()
                .with_reset()
                .invisible()
                .to_string(),
        )
    }
}

impl Icon for Venv {
    fn icon(&self, mode: IconMode) -> &'static str {
        use IconMode::*;
        match &mode {
            Text => "py",
            Icons | MinimalIcons => "",
        }
    }
}

fn venv_name(path: &Path) -> &str {
    path.ancestors()
        .filter_map(Path::file_name)
        .filter_map(OsStr::to_str)
        .find(|name| !["venv", "env", "virtualenv"].contains(name))
        .map_or("<venv>", |name| {
            ["venv", "virtualenv", "env", "-", "_"]
                .iter()
                .fold(name, |s, suf| s.strip_suffix(suf).unwrap_or(s))
        })
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
                    .to_owned(),
            )
        }))
}
