//! Default top and bottom statuslines with default title generator

use crate::{BlockType, Environment, Extend, IconMode, Pretty, Style as _};
use std::borrow::Cow;

/// Default top part of statusline
#[must_use]
pub fn top(env: &Environment) -> [Box<dyn Extend>; 11] {
    [
        BlockType::HostUser,
        BlockType::Ssh,
        BlockType::GitRepo,
        BlockType::GitTree,
        BlockType::BuildInfo,
        BlockType::Venv,
        BlockType::Jobs,
        BlockType::Mail,
        BlockType::Workdir,
        BlockType::Elapsed,
        BlockType::Time,
    ]
    .map(|x| x.create_from_env(env))
}

/// Default top line extender
pub fn extend<const N: usize>(top: [Box<dyn Extend>; N]) -> [Box<dyn Pretty>; N] {
    top.map(Extend::extend)
}

/// Default bottom part of statusline
///
/// Immutable, intended to use in `readline`-like functions
#[must_use]
pub fn bottom(env: &Environment) -> [Box<dyn Extend>; 3] {
    [
        BlockType::ReturnCode,
        BlockType::RootShell,
        BlockType::Separator,
    ]
    .map(|x| x.create_from_env(env))
}

/// Default title for statusline
///
/// Shows username, hostname and current working directory
#[must_use]
pub fn title(env: &Environment) -> String {
    let pwd = if let Some((home, user)) = &env.current_home {
        let wd = env
            .work_dir
            .strip_prefix(home)
            .unwrap_or(&env.work_dir)
            .to_str()
            .unwrap_or("<path>");
        Cow::from(if wd.is_empty() {
            format!("~{user}")
        } else {
            format!("~{user}/{wd}")
        })
    } else {
        Cow::from(env.work_dir.to_str().unwrap_or("<path>"))
    };
    format!("{}@{}: {}", env.user, env.host, pwd)
        .as_title()
        .to_string()
}

/// Default pretty-printer
#[must_use]
pub fn pretty<T: Pretty + ?Sized, const N: usize>(line: &[Box<T>; N], mode: &IconMode) -> String {
    line.iter()
        .filter_map(|x| x.as_ref().pretty(mode))
        .collect::<Vec<_>>()
        .join(" ")
}
