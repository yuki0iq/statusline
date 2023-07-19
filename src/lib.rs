#![feature(fs_try_exists)]
#![feature(let_chains)]

mod file;
mod git;
mod prompt;
mod style;
mod time;

use crate::file::{file_exists, file_exists_that, find_current_home, get_hostname, upfind};
use crate::git::{git_info, GitStatus};
use crate::prompt::PromptMode;
use crate::style::*;
use chrono::prelude::*;
use const_format::concatcp;
use nix::unistd::{access, getuid, AccessFlags};
use std::{
    env, fmt,
    path::{Path, PathBuf},
};
use time::microseconds_to_string;

fn buildinfo(workdir: &Path) -> String {
    let mut res = Vec::new();
    if file_exists("CMakeLists.txt") {
        res.push("cmake");
    }
    if file_exists("configure") {
        res.push("./configure");
    }
    if file_exists("Makefile") {
        res.push("make");
    }
    if file_exists("install") {
        res.push("./install");
    }
    if file_exists("jr") {
        res.push("./jr");
    }
    if file_exists_that(|filename| filename.ends_with(".qbs")) {
        res.push("qbs");
    }
    if file_exists_that(|filename| filename.ends_with(".pro")) {
        res.push("qmake");
    }
    if upfind(workdir, "Cargo.toml").is_some() {
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
        let jobs_count = arg.get(1).map(|val| val.as_ref().parse().unwrap()).unwrap_or(0);
        let elapsed_time = arg.get(2).map(|val| val.as_ref().parse().unwrap());
        CommandLineArgs {
            ret_code,
            jobs_count,
            elapsed_time,
        }
    }
}

pub struct StatusLine {
    prompt_mode: PromptMode,
    hostname: String,
    read_only: bool,
    git: Option<(PathBuf, GitStatus)>,
    current_home: Option<(PathBuf, String)>,
    build_info: String,
    workdir: PathBuf,
    username: String,
    is_root: bool,
    args: CommandLineArgs,
}

impl StatusLine {
    pub fn from_env<T: AsRef<str>>(args: &[T]) -> Self {
        let username = env::var("USER").unwrap_or_else(|_| String::from("<user>"));
        let workdir = env::current_dir().unwrap_or_else(|_| PathBuf::new());
        let read_only = access(&workdir, AccessFlags::W_OK).is_err();
        StatusLine {
            prompt_mode: PromptMode::new(),
            hostname: get_hostname(),
            read_only,
            git: git_info(&workdir),
            current_home: find_current_home(&workdir, &username),
            build_info: buildinfo(&workdir),
            workdir,
            username,
            is_root: getuid().is_root(),
            args: CommandLineArgs::from_env(args),
        }
    }

    fn get_workdir_str(&self) -> String {
        let (middle, highlighted) = match (&self.git, &self.current_home) {
            (Some((git_root, _)), Some((home_root, _))) => {
                if home_root.starts_with(git_root) {
                    (None, self.workdir.strip_prefix(home_root).ok())
                } else {
                    (
                        git_root.strip_prefix(home_root).ok(),
                        self.workdir.strip_prefix(git_root).ok(),
                    )
                }
            }
            (Some((git_root, _)), None) => (
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
}

impl fmt::Display for StatusLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let host_str = format!(
            "{STYLE_BOLD}{COLOR_YELLOW}({} {}{STYLE_BOLD}{COLOR_YELLOW}){STYLE_RESET}",
            self.prompt_mode.host_text(),
            colorize(&self.hostname)
        );
        let user_str = format!(
            "{STYLE_BOLD}{COLOR_BLUE}[{} {}{STYLE_BOLD}{COLOR_BLUE}]{STYLE_RESET}",
            self.prompt_mode.user_text(),
            colorize(&self.username)
        );
        let hostuser = format!("{host_str} {user_str}");

        let workdir = self.get_workdir_str();
        let readonly = if self.read_only {
            format!(
                "{}{}{}",
                COLOR_RED,
                self.prompt_mode.read_only(),
                STYLE_RESET
            )
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

        let root_str = format!(
            "{STYLE_BOLD}{}{STYLE_RESET}",
            if self.is_root {
                concatcp!(COLOR_RED, "#")
            } else {
                concatcp!(COLOR_GREEN, "$")
            },
        );

        let datetime = Local::now()
            .format("%a, %Y-%b-%d, %H:%M:%S in %Z")
            .to_string();

        let gitinfo = if let Some((_, git_status)) = &self.git {
            format!(
                "{STYLE_BOLD}{COLOR_PINK}[{} {}]{STYLE_RESET}",
                self.prompt_mode.on_branch(),
                git_status
            )
        } else {
            String::new()
        };

        let returned = match &self.args.ret_code {
            Some(0) | Some(130) => format!(
                "{COLOR_LIGHT_GREEN}{}{STYLE_RESET}",
                self.prompt_mode.return_ok()
            ),
            Some(_) => format!(
                "{COLOR_LIGHT_RED}{}{STYLE_RESET}",
                self.prompt_mode.return_fail()
            ),
            None => format!(
                "{COLOR_GREY}{}{STYLE_RESET}",
                self.prompt_mode.return_unavailable()
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

        let elapsed =
            if let Some(formatted) = self.args.elapsed_time.and_then(microseconds_to_string) {
                format!(
                    "{COLOR_CYAN}({} {}){STYLE_RESET}",
                    self.prompt_mode.took_time(),
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
        let top_line = format!(
            "{INVISIBLE_START}{}{ESC}[{}G{COLOR_GREY}{}{STYLE_RESET}{INVISIBLE_END}",
            top_left_line,
            term_size::dimensions().map(|s| s.0).unwrap_or(80) as i32 - datetime.len() as i32,
            datetime,
        );

        let bottom_line = autojoin(&[&jobs, &returned, &root_str], " ");

        write!(
            f,
            "{}\n{}\n{} ",
            title(&self.workdir.to_string_lossy()),
            top_line,
            bottom_line
        )
    }
}
