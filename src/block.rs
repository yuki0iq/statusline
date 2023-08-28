use crate::{Environment, FromEnv, Pretty};

pub mod jobs;
pub mod return_code;
pub mod root_shell;

/// All available statusline block types
pub enum BlockType {
    /// Show background jobs count
    Jobs,
    /// Show simple return code (success or failure)
    ReturnCode,
    /// Show `#` instead of `$` if root
    RootShell,
    // TODO blocktypes
    //HostUser,
    //Git,
    //BuildInfo(buildinfo::BuildInfo),
    //Venv,
    //Workdir,
    //Elapsed,
    //DateTime,
}

impl BlockType {
    /// Creates a block from given environment. These blocks can only be pretty-printed
    // TODO blocktype "trait Extend"
    pub fn create_from_env(&self, env: &Environment) -> Box<dyn Pretty> {
        match &self {
            Self::Jobs => Box::new(jobs::Jobs::from_env(env)),
            Self::ReturnCode => Box::new(return_code::ReturnCode::from_env(env)),
            Self::RootShell => Box::new(root_shell::RootShell::from_env(env)),
        }
    }
}
