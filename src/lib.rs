#![feature(io_error_more)]
#![feature(fs_try_exists)]
#![feature(let_chains)]
#![feature(slice_first_last_chunk)]
#![feature(stdsimd)]

pub mod chassis;
pub mod file;
pub mod git;
pub mod prompt;
pub mod style;
pub mod time;
pub mod virt;

use crate::git::{GitStatus, GitStatusExtended};
use crate::prompt::Prompt;
use crate::style::*;
use chrono::prelude::*;
use const_format::concatcp;
use nix::unistd::{self, AccessFlags};
use std::{
    env,
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

struct CommandLineArgs {
    ret_code: Option<u8>,
    jobs_count: u16,
    elapsed_time: Option<u64>,
}

impl CommandLineArgs {
    fn from_env<T: AsRef<str>>(arg: &[T]) -> CommandLineArgs {
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

pub struct StatusLine {
    prompt: Prompt,
    hostname: String,
    read_only: bool,
    git: Option<GitStatus>,
    git_ext: Option<GitStatusExtended>,
    current_home: Option<(PathBuf, String)>,
    build_info: String,
    workdir: PathBuf,
    username: String,
    is_root: bool,
    args: CommandLineArgs,
    is_ext: bool,
}

impl StatusLine {
    pub fn from_env<T: AsRef<str>>(args: &[T]) -> Self {
        let username = env::var("USER").unwrap_or_else(|_| String::from("<user>"));
        let workdir = env::current_dir().unwrap_or_else(|_| PathBuf::new());
        let read_only = unistd::access(&workdir, AccessFlags::W_OK).is_err();
        StatusLine {
            prompt: Prompt::build(),
            hostname: file::get_hostname(),
            read_only,
            git: GitStatus::build(&workdir).ok(),
            git_ext: None,
            current_home: file::find_current_home(&workdir, &username),
            build_info: buildinfo(&workdir),
            workdir,
            username,
            is_root: unistd::getuid().is_root(),
            args: CommandLineArgs::from_env(args),
            is_ext: false,
        }
    }

    pub fn extended(self) -> Self {
        StatusLine {
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

        let home_str = if let Some((_, user)) = &self.current_home {
            format!("{STYLE_BOLD}{COLOR_YELLOW}~{}{STYLE_RESET}", user)
        } else {
            String::new()
        };

        let middle_str = if let Some(middle) = middle {
            String::from(middle.to_string_lossy())
        } else {
            String::new()
        };

        let highlighted_str = if let Some(highlighted) = highlighted {
            let highlighted = highlighted.to_string_lossy();
            format!("{COLOR_CYAN}/{}{STYLE_RESET}", highlighted)
        } else {
            String::new()
        };

        autojoin(&[&home_str, &middle_str], "/") + &highlighted_str
    }

    pub fn to_top(&self) -> String {
        let user_str = format!(
            "{STYLE_BOLD}{}{} {}",
            self.prompt.hostuser_left(),
            self.prompt.user_text(),
            self.username
        );
        let host_str = format!(
            "{STYLE_BOLD}{}{} {}{}",
            self.prompt.hostuser_at(),
            self.hostname,
            self.prompt.host_text(),
            self.prompt.hostuser_right()
        );
        let hostuser = format!(
            "{}{}",
            colorize(&self.username, &user_str),
            colorize(&self.hostname, &host_str),
        );

        let workdir = self.get_workdir_str();
        let readonly = if self.read_only {
            format!("{}{}{}", COLOR_RED, self.prompt.read_only(), STYLE_RESET)
        } else {
            String::new()
        };

        let buildinfo = if !self.build_info.is_empty() {
            format!(
                "{STYLE_BOLD}{COLOR_PURPLE}[{}]{STYLE_RESET}",
                self.build_info
            )
        } else {
            String::new()
        };

        let datetime = Local::now()
            .format("%a, %Y-%b-%d, %H:%M:%S in %Z")
            .to_string();

        let gitinfo = if let Some(git_status) = &self.git {
            format!(
                "{STYLE_BOLD}{COLOR_PINK}[{}{}]{STYLE_RESET}",
                git_status.pretty(&self.prompt),
                if self.is_ext {
                    self.git_ext
                        .as_ref()
                        .map(|x| x.pretty(&self.prompt))
                        .unwrap_or_default()
                } else {
                    "...".to_string()
                }
            )
        } else {
            String::new()
        };

        let elapsed = if let Some(formatted) = self
            .args
            .elapsed_time
            .and_then(time::microseconds_to_string)
        {
            format!(
                "{COLOR_CYAN}({} {}){STYLE_RESET}",
                self.prompt.took_time(),
                &formatted
            )
        } else {
            String::new()
        };

        let top_left_line = autojoin(
            &[
                &hostuser, &gitinfo, &buildinfo, &readonly, &workdir, &elapsed,
            ],
            " ",
        );

        format!(
            "{INVISIBLE_START}{}{}{ESC}[{}G{COLOR_GREY}{}{STYLE_RESET}{INVISIBLE_END}",
            top_left_line,
            (if self.is_ext { "   " } else { "" }),
            term_size::dimensions().map(|s| s.0).unwrap_or(80) as i32 - datetime.len() as i32,
            datetime,
        )
    }

    pub fn to_bottom(&self) -> String {
        let root_str = format!(
            "{STYLE_BOLD}{}{STYLE_RESET}",
            if self.is_root {
                concatcp!(COLOR_RED, "#")
            } else {
                concatcp!(COLOR_GREEN, "$")
            },
        );

        let returned = match &self.args.ret_code {
            Some(0) | Some(130) => format!(
                "{COLOR_LIGHT_GREEN}{}{STYLE_RESET}",
                self.prompt.return_ok()
            ),
            Some(_) => format!(
                "{COLOR_LIGHT_RED}{}{STYLE_RESET}",
                self.prompt.return_fail()
            ),
            None => format!(
                "{COLOR_GREY}{}{STYLE_RESET}",
                self.prompt.return_unavailable()
            ),
        };

        let jobs = if self.args.jobs_count != 0 {
            format!(
                "{STYLE_BOLD}{COLOR_GREEN}[{} {}]{STYLE_RESET}",
                self.args.jobs_count,
                if self.args.jobs_count == 1 {
                    "job"
                } else {
                    "jobs"
                }
            )
        } else {
            String::new()
        };

        let bottom_line = autojoin(&[&jobs, &returned, &root_str], " ");

        format!("{} ", bottom_line)
    }

    pub fn to_title(&self, prefix: &str) -> String {
        let pwd = self.workdir.to_str().unwrap_or("<path>");
        let extended = format!("{}: {}", prefix, pwd);
        title(if prefix.is_empty() { pwd } else { &extended })
    }
}
