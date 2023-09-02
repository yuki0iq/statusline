use crate::{file, Environment, Icon, IconMode, Pretty, SimpleBlock, Style};
use nix::unistd::{self, AccessFlags};
use std::path::{Path, PathBuf};

pub struct Workdir {
    work_dir: PathBuf,
    git_tree: Option<PathBuf>,
    current_home: Option<(PathBuf, String)>,
    read_only: bool,
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
        let current_home = file::find_current_home(&work_dir, &env.user);
        let read_only = unistd::access(&work_dir, AccessFlags::W_OK).is_err();

        Workdir {
            work_dir,
            git_tree,
            current_home,
            read_only,
        }
    }
}

impl Icon for Workdir {
    fn icon(&self, mode: &IconMode) -> &'static str {
        use IconMode::*;
        match mode {
            Text => "R/O",
            Icons | MinimalIcons => "",
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
                .yellow()
                .bold()
                .with_reset()
                .to_string()
        });

        let middle_str = middle.and_then(Path::to_str).map(ToString::to_string);

        let highlighted_str = highlighted
            .and_then(Path::to_str)
            .map(|s| format!("/{}", s).cyan().with_reset().to_string());

        // eprintln!("home: {home_str:?} | middle: {middle_str:?}\n");
        let work_dir = [home_str, middle_str]
            .into_iter()
            .filter(|x| matches!(x, Some(q) if !q.is_empty()))
            .map(Option::unwrap)
            .collect::<Vec<_>>()
            .join("/")
            + &highlighted_str.unwrap_or_default();

        let read_only = self.icon(mode).red().with_reset().to_string();

        let formatted = if self.read_only {
            format!("{} {}", read_only, work_dir)
        } else {
            work_dir
        };

        Some(formatted)
    }
}
