use crate::{Environment, Icon, IconMode, Pretty, SimpleBlock, Style};
use nix::{
    sys::stat,
    unistd::{self, AccessFlags},
};
use std::{
    borrow::Cow,
    fs,
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
    fn icon(&self, mode: &IconMode) -> &'static str {
        use IconMode::*;
        match self {
            Self::Writeable => "",
            Self::Readable => match mode {
                Text => "R/O ",
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
    fn pretty(&self, mode: &IconMode) -> Option<String> {
        Some(self.icon(mode).red().with_reset().to_string())
    }
}

fn get_state(work_dir: &Path) -> (Cow<Path>, State) {
    let cwd = Cow::from(work_dir);

    let Ok(stat_dot) = stat::stat(".") else {
        return (cwd, State::NoAccess);
    };
    if 0 == stat_dot.st_nlink {
        let ret_cwd_del = (cwd, State::Deleted);

        // If workdir is deleted, then rust's `env::get_working_dir()` returns error and I fall back
        // to using $PWD, which, in some cases, is wrong. If there is a _new_ directory with path
        // same as $PWD, but not same as real cwd is, `cd .` changes workdir to $PWD, but `cd ..`
        // changes workdir to `(cwd)/..` AND updates $PWD. This seems illogical, and, probably, is.
        // Real path can be found, at least on Linux, under `/proc/self/cwd`, and it WILL contain
        // ` (deleted)` suffix in it (regular folders can also have this sequence in their names,
        // so this suffix can only be used for displaying path, and not for detecting deleted cwd).
        // What a hell.

        let Ok(cwd_del) = fs::read_link("/proc/self/cwd") else {
            return ret_cwd_del; // This may be wrong...
        };

        let cwd_del = cwd_del.into_os_string();
        let path = cwd_del.to_string_lossy();
        let deleted = " (deleted)";
        let len = path.len().saturating_sub(deleted.len());
        if &path[len..] != deleted {
            return ret_cwd_del;
        }

        let cwd_del = PathBuf::from(&path[..len]);
        return (Cow::from(cwd_del), State::Deleted);
    }

    let Ok(stat_pwd) = stat::stat(work_dir) else {
        return (cwd, State::Moved);
    };
    (
        cwd,
        if (stat_dot.st_dev, stat_dot.st_ino) != (stat_pwd.st_dev, stat_pwd.st_ino) {
            State::Moved
        } else if work_dir.ne(&PathBuf::from(std::env::var("PWD").unwrap_or_default())) {
            State::Moved
        } else if unistd::access(work_dir, AccessFlags::W_OK).is_err() {
            State::Readable
        } else {
            State::Writeable
        },
    )
}

pub struct Workdir {
    work_dir: PathBuf,
    git_tree: Option<PathBuf>,
    current_home: Option<(PathBuf, String)>,
    state: State,
}

impl SimpleBlock for Workdir {
    fn extend(self: Box<Self>) -> Box<dyn Pretty> {
        self
    }
}

impl From<&Environment> for Workdir {
    fn from(env: &Environment) -> Self {
        let work_dir = env.work_dir.clone();
        let git_tree = env.git_tree.clone();
        let current_home = env.current_home.clone();
        let (work_dir, state) = get_state(&work_dir);

        let work_dir = work_dir.into_owned();

        Workdir {
            work_dir,
            git_tree,
            current_home,
            state,
        }
    }
}

impl Pretty for Workdir {
    fn pretty(&self, mode: &IconMode) -> Option<String> {
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
            format!("~{}", user)
                .visible()
                .yellow()
                .bold()
                .with_reset()
                .invisible()
                .to_string()
        });

        let middle_str = middle.and_then(Path::to_str).map(ToString::to_string);

        let highlighted_str = highlighted.and_then(Path::to_str).map(|s| {
            format!("/{}", s)
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
