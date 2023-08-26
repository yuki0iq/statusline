use crate::{
    file, time, venv::Venv, Environment, FromEnv, GitStatus, GitStatusExtended, Icon, Icons,
    Pretty, Style,
};
use chrono::prelude::*;
use nix::unistd::{self, AccessFlags};
use std::{
    env,
    ops::Not,
    path::{Path, PathBuf},
};

fn buildinfo(workdir: &Path) -> String {
    let mut res = Vec::new();
    if file::exists("CMakeLists.txt") {
        res.push("cmake");
    }
    if file::exists("configure") {
        res.push("./configure");
    }
    if file::exists("Makefile") {
        res.push("make");
    }
    if file::exists("install") {
        res.push("./install");
    }
    if file::exists("jr") {
        res.push("./jr");
    }
    if let Ok(true) = file::exists_that(&workdir, |filename| filename.ends_with(".qbs")) {
        res.push("qbs");
    }
    if let Ok(true) = file::exists_that(&workdir, |filename| filename.ends_with(".pro")) {
        res.push("qmake");
    }
    if file::upfind(workdir, "Cargo.toml").is_ok() {
        res.push("cargo");
    }
    res.join(" ")
}

fn autojoin(vec: &[&str], sep: &str) -> String {
    vec.iter()
        .copied()
        .filter(|el| !el.is_empty())
        .collect::<Vec<&str>>()
        .join(sep)
}

/// The top part of status line
pub struct Top {
    // HostUser
    username: String,
    hostname: String,

    // Workdir
    current_home: Option<(PathBuf, String)>,
    workdir: PathBuf,
    read_only: bool,

    // Git
    git: Option<GitStatus>,
    git_ext: Option<GitStatusExtended>,

    // Buildinfo
    build_info: String,

    // Elapsed
    elapsed_time: Option<u64>,

    // Python venv
    venv: Option<Venv>,

    // Shared:
    // - git tree (Option PathBuf)
    is_ext: bool,
}

impl FromEnv for Top {
    /// Creates top statusline from environment variables and command line arguments (return code,
    /// jobs count and elapsed time in microseconds).
    ///
    /// The statusline created is __basic__ --- it only knows the information which can be
    /// acquired fast. Currently, the only slow information is full git status.
    // TODO use enviromnent
    fn from_env(args: &Environment) -> Self {
        let username = env::var("USER").unwrap_or_else(|_| String::from("<user>"));
        let workdir = args.work_dir.clone();
        let read_only = unistd::access(&workdir, AccessFlags::W_OK).is_err();
        Self {
            hostname: file::get_hostname(),
            read_only,
            git: GitStatus::build(&workdir).ok(),
            git_ext: None,
            current_home: file::find_current_home(&workdir, &username),
            build_info: buildinfo(&workdir),
            workdir,
            username,
            elapsed_time: args.elapsed_time,
            venv: Venv::get(),
            is_ext: false,
        }
    }
}

impl Top {
    /// Extends the statusline.
    ///
    /// This queries "slow" information, which is currently a git status.
    pub fn extended(self) -> Self {
        Top {
            is_ext: true,
            git_ext: self.git.as_ref().and_then(|st| st.extended()),
            ..self
        }
    }

    fn get_workdir_str(&self) -> String {
        let (middle, highlighted) = match (&self.git, &self.current_home) {
            (Some(GitStatus { tree: git_root, .. }), Some((home_root, _))) => {
                if home_root.starts_with(git_root) {
                    (None, self.workdir.strip_prefix(home_root).ok())
                } else {
                    (
                        git_root.strip_prefix(home_root).ok(),
                        self.workdir.strip_prefix(git_root).ok(),
                    )
                }
            }
            (Some(GitStatus { tree: git_root, .. }), None) => (
                Some(git_root.as_path()),
                self.workdir.strip_prefix(git_root).ok(),
            ),
            (None, Some((home_root, _))) => (self.workdir.strip_prefix(home_root).ok(), None),
            (None, None) => (Some(self.workdir.as_path()), None),
        };

        let home_str = self
            .current_home
            .as_ref()
            .map(|(_, user)| {
                format!("~{}", user)
                    .yellow()
                    .bold()
                    .with_reset()
                    .to_string()
            })
            .unwrap_or_default();
        let middle_str = middle
            .and_then(Path::to_str)
            .map(ToString::to_string)
            .unwrap_or_default();
        let highlighted_str = highlighted
            .and_then(Path::to_str)
            .map(|s| format!("/{}", s).cyan().with_reset().to_string())
            .unwrap_or_default();

        autojoin(&[&home_str, &middle_str], "/") + &highlighted_str
    }

    /// Format the title for terminal.
    pub fn to_title(&self, prefix: Option<&str>) -> String {
        let pwd = self.workdir.to_str().unwrap_or("<path>");
        let prefix = prefix
            .map(|p| format!("{} ", p.boxed()))
            .unwrap_or_default();
        format!("{}{}@{}: {}", prefix, self.username, self.hostname, pwd)
            .as_title()
            .to_string()
    }
}

impl Pretty for Top {
    /// Format the top part of statusline.
    fn pretty(&self, icons: &Icons) -> Option<String> {
        let user_str = format!("[{} {}", icons(Icon::User), self.username);
        let host_str = format!(
            "{}{} {}]",
            icons(Icon::HostAt),
            self.hostname,
            icons(Icon::Host),
        );
        let hostuser = format!(
            "{}{}",
            user_str.colorize_with(&self.username),
            host_str.colorize_with(&self.hostname)
        )
        .bold()
        .with_reset()
        .to_string();

        let workdir = self.get_workdir_str();
        let readonly = self
            .read_only
            .then_some(icons(Icon::ReadOnly).red().with_reset().to_string())
            .unwrap_or_default();

        let buildinfo = self
            .build_info
            .is_empty()
            .not()
            .then_some(
                self.build_info
                    .boxed()
                    .purple()
                    .bold()
                    .with_reset()
                    .to_string(),
            )
            .unwrap_or_default();

        let datetime_str = Local::now()
            .format("%a, %Y-%b-%d, %H:%M:%S in %Z")
            .to_string();
        let term_width = term_size::dimensions().map(|s| s.0).unwrap_or(80) as i32;
        let datetime = datetime_str
            .gray()
            .with_reset()
            .horizontal_absolute(term_width - datetime_str.len() as i32)
            .to_string();

        let gitinfo = self
            .git
            .as_ref()
            .map(|git_status| {
                (git_status.pretty(icons)
                    + &self
                        .is_ext
                        .then_some(
                            self.git_ext
                                .as_ref()
                                .map(|x| x.pretty(icons))
                                .unwrap_or_default(),
                        )
                        .unwrap_or("...".to_string()))
                    .boxed()
                    .pink()
                    .bold()
                    .with_reset()
                    .to_string()
            })
            .unwrap_or_default();

        let elapsed = self
            .elapsed_time
            .and_then(time::microseconds_to_string)
            .map(|ms| {
                format!("{} {}", icons(Icon::TookTime), ms)
                    .rounded()
                    .cyan()
                    .with_reset()
                    .to_string()
            })
            .unwrap_or_default();

        let pyvenv = self
            .venv
            .as_ref()
            .map(|venv| {
                venv.pretty(icons)
                    .boxed()
                    .yellow()
                    .bold()
                    .with_reset()
                    .to_string()
            })
            .unwrap_or_default();

        let top_left_line = autojoin(
            &[
                &hostuser, &gitinfo, &pyvenv, &buildinfo, &readonly, &workdir, &elapsed,
            ],
            " ",
        );

        Some(format!(
            "{}{}{}",
            top_left_line,
            (if self.is_ext { "   " } else { "" }),
            datetime,
        ))
    }
}
