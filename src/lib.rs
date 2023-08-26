//! Status line for shells with ANSI escape sequences support
//!
//! This is a documentation for statusline API, use `README.md` for executable documentation
//!
//! # Example
//!
//! ```
//! use statusline::{Bottom, CommandLineArgs, Icons, Top};
//!
//! let icons = Icons::MinimalIcons;
//! let args = CommandLineArgs::from_env::<&str>(&[]);
//! let top = Top::from_env(&args);
//! let bottom = Bottom::from_env(&args);
//! println!("{}", top.to_title(Some("test")));
//! println!("{}", top.pretty(&icons));
//! print!("{}", bottom.pretty(&icons));  // Or you can use readline with result as prompt
//!
//! // And, additionally, you can start a separate thread for getting more info
//! // which should be outputed "over" the first top line
//! ```

#![feature(byte_slice_trim_ascii)]
#![feature(io_error_more)]
#![feature(iter_next_chunk)]
#![feature(fn_traits)]
#![feature(fs_try_exists)]
#![feature(let_chains)]
#![feature(slice_first_last_chunk)]
#![feature(stdsimd)]
#![feature(unboxed_closures)]

mod chassis;
mod git;
mod icon;
mod style;
mod time;
mod venv;

/// Filesystem-related operations
pub mod file;

/// Virtualization detector (not tested tho)
pub mod virt;

pub use crate::{
    chassis::Chassis,
    git::{GitStatus, GitStatusExtended},
    icon::{Icon, IconMode as Icons},
    style::{Style, Styled},
};

use crate::venv::Venv;
use chrono::prelude::*;
use nix::unistd::{self, AccessFlags};
use std::{
    env,
    ops::Not,
    path::{Path, PathBuf},
    string::ToString,
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

/// Parsed command line arguments
pub struct CommandLineArgs {
    /// Last command's return code
    ret_code: Option<u8>,
    /// Jobs currently running
    jobs_count: usize,
    /// Last command's elapsed tile
    elapsed_time: Option<u64>,
}

impl CommandLineArgs {
    /// Construct args from command line
    pub fn from_env<T: AsRef<str>>(arg: &[T]) -> CommandLineArgs {
        let ret_code = arg.get(0).map(|val| val.as_ref().parse().unwrap());
        let jobs_count = arg
            .get(1)
            .map(|val| val.as_ref().parse().unwrap_or(0))
            .unwrap_or(0);
        let elapsed_time = arg.get(2).map(|val| val.as_ref().parse().unwrap());
        CommandLineArgs {
            ret_code,
            jobs_count,
            elapsed_time,
        }
    }
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

/// The bottom part of statusline. Immutable, intended to use in `readline`-like functions
pub struct Bottom {
    is_root: bool,

    // Background jobs count
    jobs: usize,

    // Process return code
    return_code: Option<u8>,
}

impl Top {
    /// Creates top statusline from environment variables and command line arguments (return code,
    /// jobs count and elapsed time in microseconds).
    ///
    /// The statusline created is __basic__ --- it only knows the information which can be
    /// acquired fast. Currently, the only slow information is full git status.
    pub fn from_env(args: &CommandLineArgs) -> Self {
        let username = env::var("USER").unwrap_or_else(|_| String::from("<user>"));
        let workdir = env::current_dir().unwrap_or_else(|_| PathBuf::new());
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

    /// Format the top part of statusline.
    pub fn pretty(&self, icons: &Icons) -> String {
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

        format!(
            "{}{}{}",
            top_left_line,
            (if self.is_ext { "   " } else { "" }),
            datetime,
        )
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

impl Bottom {
    pub fn from_env(args: &CommandLineArgs) -> Self {
        Self {
            is_root: unistd::getuid().is_root(),
            jobs: args.jobs_count,
            return_code: args.ret_code,
        }
    }

    /// Format the bottom part of the statusline.
    pub fn pretty(&self, icons: &Icons) -> String {
        let root = self
            .is_root
            .then_some("#".visible().red())
            .unwrap_or("$".visible().green())
            .bold()
            .with_reset()
            .invisible()
            .to_string();

        let (ok, fail, na) = (
            icons(Icon::ReturnOk).visible(),
            icons(Icon::ReturnFail).visible(),
            icons(Icon::ReturnNA).visible(),
        );
        let returned = match &self.return_code {
            Some(0) | Some(130) => ok.light_green(),
            Some(_) => fail.light_red(),
            None => na.light_gray(),
        }
        .with_reset()
        .invisible()
        .to_string();

        let jobs = 0
            .ne(&self.jobs)
            .then_some(
                format!(
                    "{} job{}",
                    self.jobs,
                    1.ne(&self.jobs).then_some("s").unwrap_or_default()
                )
                .boxed()
                .visible()
                .green()
                .bold()
                .with_reset()
                .invisible()
                .to_string(),
            )
            .unwrap_or_default();

        let bottom_line = autojoin(&[&jobs, &returned, &root], " ");

        format!("{} ", bottom_line)
    }
}
