use pwd::Passwd;
use std::path::{Path, PathBuf};

#[must_use]
pub fn find_current_home(path: &Path, cur_user: &str) -> Option<(PathBuf, String)> {
    Passwd::iter()
        .find(|Passwd { dir, .. }| {
            dir != "/"
                && !["bin", "dev", "proc", "usr", "var"].contains(
                    &dir.strip_prefix('/')
                        .unwrap_or_default()
                        .split('/')
                        .next()
                        .unwrap_or_default(),
                )
                && path.starts_with(dir)
        })
        .map(|Passwd { name, dir, .. }| {
            (
                PathBuf::from(dir),
                if name == cur_user {
                    String::new()
                } else {
                    name
                },
            )
        })
}

pub fn exists<P: AsRef<Path>>(path: P) -> bool {
    std::fs::exists(path).unwrap_or(false)
}

pub fn points_to_file<P: AsRef<Path>>(path: P) -> bool {
    std::fs::metadata(path)
        .map(|meta| meta.is_file())
        .unwrap_or(false)
}

pub fn exists_that<P: AsRef<Path>, F>(path: P, mut f: F) -> std::io::Result<bool>
where
    F: FnMut(&str) -> bool,
{
    for entry in std::fs::read_dir(path)? {
        if entry?.file_name().to_str().is_some_and(&mut f) {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn upfind<P: AsRef<Path>>(start: P, filename: &str) -> Option<PathBuf> {
    start
        .as_ref()
        .ancestors()
        .map(|path| path.join(filename))
        .find(|path| exists(path))
}
