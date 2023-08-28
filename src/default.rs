//! Default top and bottom statuslines with default title generator
//!
//!

use crate::{BlockType, Environment, Pretty, Style};

/// Default bottom part of statusline. Immutable, intended to use in `readline`-like functions
pub fn bottom(args: &Environment) -> [Box<dyn Pretty>; 3] {
    [BlockType::Jobs, BlockType::ReturnCode, BlockType::RootShell].map(|x| x.create_from_env(args))
}

/// Default title for statusline
///
/// Shows username, hostname and current working directory
pub fn title(env: &Environment) -> String {
    let pwd = env.work_dir.to_str().unwrap_or("<path>");
    format!("{}@{}: {}", env.user, env.host, pwd)
        .as_title()
        .to_string()
}
