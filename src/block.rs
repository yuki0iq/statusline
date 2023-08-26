use crate::{Environment, FromEnv, Icons, Pretty};

pub mod jobs;
pub mod return_code;
pub mod root_shell;

pub enum BlockType {
    Jobs,
    ReturnCode,
    RootShell,
    //HostUser,
    //Git,
    //BuildInfo(buildinfo::BuildInfo),
    //Venv,
    //Workdir,
    //Elapsed,
    //DateTime,
}

impl BlockType {
    pub fn create_from_env(&self, env: &Environment) -> Box<dyn Pretty> {
        match &self {
            Self::Jobs => Box::new(jobs::Jobs::from_env(env)),
            Self::ReturnCode => Box::new(return_code::ReturnCode::from_env(env)),
            Self::RootShell => Box::new(root_shell::RootShell::from_env(env)),
        }
    }
}

pub trait Block: FromEnv + Pretty {}

impl<T: FromEnv + Pretty> Block for T {}

pub fn autopretty(vec: &[Box<dyn Pretty>], icons: &Icons, sep: &str) -> String {
    // TODO collect -- why??
    vec.iter()
        .filter_map(|x| x.as_ref().pretty(icons))
        .collect::<Vec<_>>()
        .join(sep)
}
