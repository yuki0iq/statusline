use crate::Chassis;
use std::path::PathBuf;

/// Environment variables available to statusline
pub struct Environment {
    /// Last command's return code
    pub ret_code: Option<u8>,
    /// Jobs currently running
    pub jobs_count: usize,
    /// Last command's elapsed time, in us
    pub elapsed_time: Option<u64>,
    /// Working directory
    pub work_dir: PathBuf,
    /// Git worktree path if any
    pub git_tree: Option<PathBuf>,
    /// Username
    pub user: String,
    /// Hostname
    pub host: String,
    /// Chassis
    pub chassis: Chassis,
    /// Cheernt home: dir and username
    pub current_home: Option<(PathBuf, String)>,
}
