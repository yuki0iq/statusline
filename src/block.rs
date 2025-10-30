use crate::Pretty;

mod build_info;
mod elapsed;
mod git;
mod hostuser;
mod jobs;
mod mail;
mod nix_shell;
mod return_code;
mod root_shell;
mod separator;
mod ssh;
mod time;
mod venv;
mod workdir;

pub use {
    build_info::BuildInfo,
    elapsed::Elapsed,
    git::{Repo as GitRepo, Tree as GitTree},
    hostuser::HostUser,
    jobs::Jobs,
    mail::UnseenMail,
    nix_shell::MaybeNixShell,
    return_code::ReturnCode,
    root_shell::RootShell,
    separator::Separator,
    ssh::Ssh,
    time::Time,
    venv::MaybeVenv,
    workdir::Workdir,
};

/// Simple block which can be extended (only once) and pretty-printed
pub trait Extend: Pretty {
    /// Extend block once. Many blocks remain untouched
    fn extend(self: Box<Self>) -> Box<dyn Pretty>;
}
