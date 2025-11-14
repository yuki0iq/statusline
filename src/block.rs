use crate::{Environment, Pretty};

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
    git::{GitRepo, GitTree},
    hostuser::HostUser,
    jobs::Jobs,
    mail::UnseenMail,
    nix_shell::NixShell,
    return_code::ReturnCode,
    root_shell::RootShell,
    separator::Separator,
    ssh::Ssh,
    time::Time,
    venv::Venv,
    workdir::Workdir,
};

pub trait Block: Pretty {
    fn new(environ: &Environment) -> Option<Box<dyn Block>>
    where
        Self: Sized;
    fn extend(&mut self) {}
}
