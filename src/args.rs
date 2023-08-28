use crate::file;
use std::{env, path::PathBuf};

/// Environment variables available to statusline
pub struct Environment {
    /// Last command's return code
    pub ret_code: Option<u8>,
    /// Jobs currently running
    pub jobs_count: usize,
    /// Last command's elapsed tile
    pub elapsed_time: Option<u64>,
    /// Working directory
    pub work_dir: PathBuf,
    /// Git worktree path if any
    pub git_tree: Option<PathBuf>,
}

impl Environment {
    /// Construct args from command line
    ///
    /// TODO change how arguments are passed here, this is cringe (and the code is too)
    pub fn from_env<T: AsRef<str>>(arg: &[T]) -> Self {
        let ret_code = arg.get(0).map(|val| val.as_ref().parse().unwrap());
        let jobs_count = arg
            .get(1)
            .map(|val| val.as_ref().parse().unwrap_or(0))
            .unwrap_or(0);
        let elapsed_time = arg.get(2).map(|val| val.as_ref().parse().unwrap());

        let work_dir = env::current_dir().unwrap_or_else(|_| PathBuf::new());
        let git_tree = file::upfind(&work_dir, ".git")
            .ok()
            .map(|dg| dg.parent().unwrap().to_path_buf());
        Environment {
            ret_code,
            jobs_count,
            elapsed_time,
            git_tree,
            work_dir,
        }
    }
}

/// Constructable from environment object
pub trait FromEnv {
    /// Construct object from given environment
    fn from_env(args: &Environment) -> Self;
}
