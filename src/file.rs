use pwd::Passwd;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn find_current_home(path: &Path, cur_user: &str) -> Option<(PathBuf, String)> {
    if let Some(Passwd { name, dir, .. }) = Passwd::iter().find(|Passwd { dir, .. }| {
        !(dir == "/"
            || ["bin", "dev", "proc", "usr", "var"].contains(
                &dir.strip_prefix('/')
                    .unwrap_or_default()
                    .split('/')
                    .next()
                    .unwrap_or_default(),
            ))
            && path.starts_with(&dir)
    }) {
        Some((
            PathBuf::from(dir),
            if name != cur_user {
                name
            } else {
                String::new()
            },
        ))
    } else {
        None
    }
}

pub fn file_exists<P: AsRef<Path> + ?Sized>(path: &P) -> bool {
    fs::try_exists(path.as_ref()).unwrap_or(false)
}

pub fn file_exists_that<F>(f: F) -> bool
where
    F: Fn(&str) -> bool,
{
    let Ok(dir_iter) = fs::read_dir(".") else {
        return false;
    };
    for entry_res in dir_iter {
        let Ok(entry) = entry_res else {
            return false;
        };
        if let Ok(filename) = entry.file_name().into_string() {
            if f(&filename) {
                return true;
            }
        }
    }
    false
}

pub fn upfind(start: &Path, filename: &str) -> Option<PathBuf> {
    start
        .ancestors()
        .map(|path| path.join(filename))
        .find(file_exists)
}

pub fn get_hostname() -> String {
    let hostname = fs::read_to_string("/etc/hostname").unwrap_or_else(|_| String::from("<host>"));
    String::from(hostname.trim())
}
