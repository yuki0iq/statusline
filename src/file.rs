use anyhow::{Context as _, Result};
use pwd::Passwd;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[must_use]
pub fn find_current_home(path: &Path, cur_user: &str) -> Option<(PathBuf, String)> {
    if let Some(Passwd { name, dir, .. }) = Passwd::iter().find(|Passwd { dir, .. }| {
        dir != "/"
            && !["bin", "dev", "proc", "usr", "var"].contains(
                &dir.strip_prefix('/')
                    .unwrap_or_default()
                    .split('/')
                    .next()
                    .unwrap_or_default(),
            )
            && path.starts_with(dir)
    }) {
        Some((
            PathBuf::from(dir),
            if name == cur_user {
                String::new()
            } else {
                name
            },
        ))
    } else {
        None
    }
}

pub fn exists<P: AsRef<Path> + ?Sized>(path: &P) -> bool {
    fs::exists(path.as_ref()).unwrap_or(false)
}

pub fn points_to_file<P: AsRef<Path> + ?Sized>(path: &P) -> bool {
    fs::metadata(path)
        .map(|meta| meta.is_file())
        .unwrap_or(false)
}

pub fn exists_that<F: Fn(&str) -> bool, P: AsRef<Path>>(path: P, f: F) -> std::io::Result<bool> {
    for entry in fs::read_dir(path)? {
        if let Ok(filename) = entry?.file_name().into_string()
            && f(&filename)
        {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn upfind<P: AsRef<Path>>(start: P, filename: &str) -> Result<PathBuf> {
    start
        .as_ref()
        .ancestors()
        .map(|path| path.join(filename))
        .find(exists)
        .context("upfind could not find parent")
}
