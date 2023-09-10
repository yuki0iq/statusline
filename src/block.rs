use crate::{Environment, Pretty};

pub mod build_info;
pub mod elapsed;
pub mod git;
pub mod hostuser;
pub mod jobs;
pub mod return_code;
pub mod root_shell;
pub mod separator;
pub mod time;
pub mod venv;
pub mod workdir;

/// All available statusline block types
pub enum BlockType {
    /// Empty separator
    Separator,
    /// Show background jobs count
    Jobs,
    /// Show simple return code (success or failure)
    ReturnCode,
    /// Show `#` instead of `$` if root
    RootShell,
    /// Hostname and username display
    HostUser,
    /// Git repo info
    Git,
    /// Build for ???
    BuildInfo,
    /// Python virtual environment name and version
    Venv,
    /// Working directory with username substitution, git repo path and R/O display
    Workdir,
    /// Previous task execution time
    Elapsed,
    /// Date and time
    Time,
}

impl BlockType {
    /// Creates a block from given environment. These blocks can only be pretty-printed
    // TODO blocktype "trait Extend"
    pub fn create_from_env(&self, env: &Environment) -> Box<dyn SimpleBlock> {
        match &self {
            Self::Separator => Box::new(separator::Separator()),
            Self::Jobs => Box::new(jobs::Jobs::from(env)),
            Self::ReturnCode => Box::new(return_code::ReturnCode::from(env)),
            Self::RootShell => Box::new(root_shell::RootShell::from(env)),
            Self::HostUser => Box::new(hostuser::HostUser::from(env)),
            Self::Git => Box::new(git::ResGit::from(env)),
            Self::BuildInfo => Box::new(build_info::BuildInfo::from(env)),
            Self::Venv => Box::new(venv::MaybeVenv::from(env)),
            Self::Workdir => Box::new(workdir::Workdir::from(env)),
            Self::Elapsed => Box::new(elapsed::Elapsed::from(env)),
            Self::Time => Box::new(time::Time::from(env)),
        }
    }
}

pub trait SimpleBlock: Pretty {
    fn extend(self: Box<Self>) -> Box<dyn Pretty>;
}
