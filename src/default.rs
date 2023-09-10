//! Default top and bottom statuslines with default title generator
//!
//!

use crate::{BlockType, Environment, IconMode, Pretty, SimpleBlock, Style};

/// Default top part of statusline
pub fn top(env: &Environment) -> [Box<dyn SimpleBlock>; 7] {
    [
        BlockType::HostUser,
        BlockType::Git,
        BlockType::BuildInfo,
        BlockType::Venv,
        BlockType::Workdir,
        BlockType::Elapsed,
        BlockType::Time,
    ]
    .map(|x| x.create_from_env(env))
}

/// Default top line extender
pub fn extend<const N: usize>(top: [Box<dyn SimpleBlock>; N]) -> [Box<dyn Pretty>; N] {
    top.map(SimpleBlock::extend)
}

/// Default bottom part of statusline
///
/// Immutable, intended to use in `readline`-like functions
pub fn bottom(env: &Environment) -> [Box<dyn SimpleBlock>; 4] {
    [
        BlockType::Jobs,
        BlockType::ReturnCode,
        BlockType::RootShell,
        BlockType::Separator,
    ]
    .map(|x| x.create_from_env(env))
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

/// Default pretty-printer
pub fn pretty<T: Pretty + ?Sized, const N: usize>(line: &[Box<T>; N], mode: &IconMode) -> String {
    line.iter()
        .filter_map(|x| x.as_ref().pretty(mode))
        .collect::<Vec<_>>()
        .join(" ")
}
