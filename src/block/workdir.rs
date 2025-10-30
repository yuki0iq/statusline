use crate::{Environment, Extend, Icon, IconMode, Pretty, Style as _};
use anyhow::{Context as _, Result, ensure};
use rustix::fs::{Access, Stat};
use std::{
    ffi::OsString,
    os::unix::ffi::OsStringExt as _,
    path::{Path, PathBuf},
};

enum State {
    Writeable,
    Readable,
    Moved,
    Deleted,
    NoAccess,
}

impl Icon for State {
    fn icon(&self, mode: IconMode) -> &'static str {
        use IconMode::*;
        match self {
            Self::Writeable => "",
            Self::Readable => match mode {
                Text => "readonly ",
                Icons | MinimalIcons => " ",
            },
            Self::Deleted => match mode {
                Text => "deleted ",
                Icons | MinimalIcons => "󰇾 ",
            },
            Self::Moved => match mode {
                Text => "moved ",
                Icons | MinimalIcons => " ",
            },
            Self::NoAccess => match mode {
                Text => "forbidden ",
                Icons | MinimalIcons => "󰂭 ",
            },
        }
    }
}

impl Pretty for State {
    fn pretty(&self, mode: IconMode) -> Option<String> {
        Some(
            self.icon(mode)
                .visible()
                .red()
                .italic()
                .with_reset()
                .invisible()
                .to_string(),
        )
    }
}

fn get_cwd_if_deleted() -> Option<PathBuf> {
    let mut cwd = std::fs::read_link("/proc/self/cwd")
        .ok()?
        .into_os_string()
        .into_vec();
    cwd.truncate(cwd.strip_suffix(b" (deleted)")?.len());
    Some(PathBuf::from(OsString::from_vec(cwd)))
}

fn ensure_work_dir_not_moved(work_dir: &Path, stat_dot: Stat) -> Result<()> {
    let stat_pwd = rustix::fs::stat(work_dir)?;
    ensure!((stat_dot.st_dev, stat_dot.st_ino) == (stat_pwd.st_dev, stat_pwd.st_ino));
    ensure!(*work_dir == std::env::var_os("PWD").context("No PWD")?);
    Ok(())
}

fn get_state(work_dir: &mut PathBuf) -> State {
    let Ok(stat_dot) = rustix::fs::stat(".") else {
        return State::NoAccess;
    };

    if 0 == stat_dot.st_nlink {
        // If workdir is deleted, then rust's `env::get_working_dir()` returns error and I fall back
        // to using $PWD, which, in some cases, is wrong. If there is a _new_ directory with path
        // same as $PWD, but not same as real cwd is, `cd .` changes workdir to $PWD, but `cd ..`
        // changes workdir to `(cwd)/..` AND updates $PWD. This seems illogical, and, probably, is.
        // Real path can be found, at least on Linux, under `/proc/self/cwd`, and it WILL contain
        // ` (deleted)` suffix in it (regular folders can also have this sequence in their names,
        // so this suffix can only be used for displaying path, and not for detecting deleted cwd).
        // What a hell.
        if let Some(cwd) = get_cwd_if_deleted() {
            *work_dir = cwd;
        }
        return State::Deleted;
    }

    if ensure_work_dir_not_moved(work_dir, stat_dot).is_err() {
        return State::Moved;
    }

    match rustix::fs::access(&*work_dir, Access::WRITE_OK) {
        Ok(()) => State::Writeable,
        Err(_) => State::Readable,
    }
}

pub struct Workdir {
    work_dir: PathBuf,
    git_tree: Option<PathBuf>,
    current_home: Option<(PathBuf, String)>,
    state: State,
}

impl Extend for Workdir {
    fn extend(self: Box<Self>) -> Box<dyn Pretty> {
        self
    }
}

impl Workdir {
    pub fn new(env: &Environment) -> Box<Self> {
        let mut work_dir = env.work_dir.clone();
        let git_tree = env.git_tree.clone();
        let current_home = env.current_home.clone();
        let state = get_state(&mut work_dir);
        Box::new(Workdir {
            work_dir,
            git_tree,
            current_home,
            state,
        })
    }
}

impl Pretty for Workdir {
    fn pretty(&self, mode: IconMode) -> Option<String> {
        let (middle, highlighted) = match (&self.git_tree, &self.current_home) {
            (Some(git_root), Some((home_root, _))) => {
                if home_root.starts_with(git_root) {
                    (None, self.work_dir.strip_prefix(home_root).ok())
                } else {
                    (
                        git_root.strip_prefix(home_root).ok(),
                        self.work_dir.strip_prefix(git_root).ok(),
                    )
                }
            }
            (Some(git_root), None) => (
                Some(git_root.as_path()),
                self.work_dir.strip_prefix(git_root).ok(),
            ),
            (None, Some((home_root, _))) => (self.work_dir.strip_prefix(home_root).ok(), None),
            (None, None) => (Some(self.work_dir.as_path()), None),
        };

        let home_str = self.current_home.as_ref().map(|(_, user)| {
            format!("~{user}")
                .visible()
                .yellow()
                .bold()
                .with_reset()
                .invisible()
                .to_string()
        });

        let middle_str = middle.and_then(Path::to_str).map(ToString::to_string);

        let highlighted_str = highlighted.and_then(Path::to_str).map(|s| {
            format!("/{s}")
                .visible()
                .cyan()
                .with_reset()
                .invisible()
                .to_string()
        });

        let work_dir = [home_str, middle_str]
            .into_iter()
            .filter(|x| matches!(x, Some(q) if !q.is_empty()))
            .map(Option::unwrap)
            .collect::<Vec<_>>()
            .join("/")
            + &highlighted_str.unwrap_or_default();

        Some(format!("{}{}", self.state.pretty(mode).unwrap(), work_dir))
    }
}
