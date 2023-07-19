use crate::file::upfind;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

pub fn find_git_root(workdir: &Path) -> Option<PathBuf> {
    Some(upfind(workdir, ".git")?.parent()?.to_path_buf())
}

pub struct GitStatus {
    branch: String,
    remote_branch: Option<String>,
    behind: u32,
    ahead: u32,
    stashes: u32,
    unmerged: u32,
    staged: u32,
    unstaged: u32,
    untracked: u32,
}

impl GitStatus {
    pub fn build() -> Option<GitStatus> {
        let out = Command::new("git")
            .args(["status", "--porcelain=2", "--branch", "--show-stash"])
            .output()
            .ok()?;
        None
    }
}
