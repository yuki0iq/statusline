use crate::{Environment, Pretty};

pub mod build_info;
pub mod elapsed;
pub mod git;
pub mod hostuser;
pub mod jobs;
pub mod mail;
pub mod nix_shell;
pub mod return_code;
pub mod root_shell;
pub mod separator;
pub mod ssh;
pub mod time;
pub mod venv;
pub mod workdir;

/// All available statusline block types
#[non_exhaustive]
pub enum Kind {
    /// Empty separator
    Separator,
    /// Continuation arrow
    Continue,
    /// Show background jobs count
    Jobs,
    /// Show simple return code (success or failure)
    ReturnCode,
    /// Show `#` instead of `$` if root
    RootShell,
    /// Hostname and username display
    HostUser,
    /// Git repo info
    GitRepo,
    /// Git tree info
    GitTree,
    /// Build for ???
    BuildInfo,
    /// Python virtual environment name and version
    Venv,
    /// Nix shell
    NixShell,
    /// Working directory with username substitution, git repo path and R/O display
    Workdir,
    /// Previous task execution time
    Elapsed,
    /// Date and time
    Time,
    /// If over ssh, show "from" connection
    Ssh,
    /// Unseen mail count
    Mail,
}

impl Kind {
    /// Creates a block from given environment. These blocks can be pretty-printed and extended
    #[must_use]
    pub fn create_from_env(&self, env: &Environment) -> Box<dyn Extend> {
        match &self {
            Self::Separator => Box::new(separator::Separator("")),
            Self::Continue => Box::new(separator::Separator("\u{f105}")),
            Self::Jobs => Box::new(jobs::Jobs::from(env)),
            Self::ReturnCode => Box::new(return_code::ReturnCode::from(env)),
            Self::RootShell => Box::new(root_shell::RootShell::from(env)),
            Self::HostUser => Box::new(hostuser::HostUser::from(env)),
            Self::GitRepo => Box::new(git::Repo::from(env)),
            Self::GitTree => Box::new(git::Tree::from(env)),
            Self::BuildInfo => Box::new(build_info::BuildInfo::from(env)),
            Self::Venv => Box::new(venv::MaybeVenv::from(env)),
            Self::NixShell => Box::new(nix_shell::MaybeNixShell::from(env)),
            Self::Workdir => Box::new(workdir::Workdir::from(env)),
            Self::Elapsed => Box::new(elapsed::Elapsed::from(env)),
            Self::Time => Box::new(time::Time::from(env)),
            Self::Ssh => Box::new(ssh::Ssh::from(env)),
            Self::Mail => Box::new(mail::UnseenMail::from(env)),
        }
    }
}
/// Simple block which can be extended (only once) and pretty-printed
pub trait Extend: Pretty {
    /// Extend block once. Many blocks remain untouched
    fn extend(self: Box<Self>) -> Box<dyn Pretty>;
}
