#![feature(fs_try_exists)]

use const_format::{concatcp, formatcp};
use nix::unistd::{access, getuid, AccessFlags};
use pwd::Passwd;
use regex::Regex;
use std::{
    env, fmt, fs,
    path::{Path, PathBuf},
};

const INVISIBLE_START: &str = "\x01";
const INVISIBLE_END: &str = "\x02";
const ESC: &str = "\x1b";
const BEL: &str = "\x07";

macro_rules! invisible {
    ( $x: expr ) => {
        concatcp!(INVISIBLE_START, $x, INVISIBLE_END)
    };
}

macro_rules! esc {
    ( $x: expr ) => {
        concatcp!(ESC, $x)
    };
}

const CSI: &str = esc!("[");

macro_rules! csi {
    ( $x: expr ) => {
        invisible!(concatcp!(CSI, $x))
    };
}

macro_rules! style {
    ( $x: expr ) => {
        csi!(concatcp!($x, "m"))
    };
}

macro_rules! color {
    ( $x: expr ) => {
        style!($x)
    };
}

macro_rules! rgb {
    ( $r: expr, $g: expr, $b: expr ) => {
        style!(formatcp!("38;2;{};{};{}", $r, $g, $b))
    };
}

// TODO compile-time
fn rgb(r: u8, g: u8, b: u8) -> String {
    format!("{INVISIBLE_START}{CSI}38;2;{r};{g};{b}m{INVISIBLE_END}")
}

const STYLE_RESET: &str = color!(0u8);
const STYLE_BOLD: &str = color!(1u8);
const COLOR_RED: &str = color!(31u8);
const COLOR_GREEN: &str = color!(32u8);
const COLOR_YELLOW: &str = color!(33u8);
const COLOR_PURPLE: &str = color!(35u8);
const COLOR_CYAN: &str = color!(36u8);
const COLOR_BLUE: &str = rgb!(0u8, 127u8, 240u8);
const COLOR_PINK: &str = rgb!(255u8, 100u8, 203u8);
const COLOR_PY_YELLOW: &str = rgb!(255u8, 219u8, 59u8);
const COLOR_LIGHT_GREEN: &str = rgb!(100u8, 255u8, 100u8);
const COLOR_LIGHT_RED: &str = rgb!(255u8, 80u8, 100u8);
const COLOR_GREY: &str = rgb!(128u8, 128u8, 128u8);

macro_rules! title {
    ( $x: expr ) => {
        invisible!(concatcp!(ESC, "]0;", $x, BEL))
    };
}

const CURSOR_SAVE: &str = esc!("[s");
const CURSOR_RESTORE: &str = esc!("[u");
const CURSOR_UP: &str = esc!("[A");
const CURSOR_HOME: &str = esc!("[G");

/*
How to "colorize" a string

using colors table
   F   C   A   6   0
 255 203 153 100   0
 -> scientific "pick" moment

allow (block too dark and too red)
FCA60 FCA6 FCA60
FCA60 0 FCA

block (r06 r00)

total 115 of 5^3 = 125, ban 10 highest
store in BGR (ban 00r, 06r)

then for hash from 0 to 114
B = hash / 25
G = hash / 5 % 5
R = hash % 5

COLOR_TABLE=(255 203 153 100 0)
*/
const COLOR_TABLE: [u8; 5] = [255, 203, 153, 100, 0];
fn colorize(s: &str) -> String {
    if s == "root" {
        String::from(concatcp!(STYLE_BOLD, COLOR_RED, "root", STYLE_RESET))
    } else {
        let hash =
            usize::from_str_radix(&sha256::digest(format!("{}\n", s))[..4], 16).unwrap() % 115;
        let b = COLOR_TABLE[hash / 25];
        let g = COLOR_TABLE[hash / 5 % 5];
        let r = COLOR_TABLE[hash % 5];
        format!("{}{s}{STYLE_RESET}", rgb(r, g, b))
    }
}

fn find_current_home(path: &Path, cur_user: &str) -> Option<(PathBuf, String)> {
    let invalid_homes = Regex::new(r"^/$|^(/bin|/dev|/proc|/usr|/var)(/|$)").unwrap();
    if let Some(Passwd { name, dir, .. }) = Passwd::iter()
        .find(|passwd| !invalid_homes.is_match(&passwd.dir) && path.starts_with(&passwd.dir))
    {
        Some((
            PathBuf::from(dir),
            if name != cur_user {
                name
            } else {
                String::new()
            },
        ))
    } else {
        None
    }
}

fn file_exists<P: AsRef<Path> + ?Sized>(path: &P) -> bool {
    fs::try_exists(path.as_ref()).unwrap_or(false)
}

fn file_exists_that<F>(f: F) -> bool
where
    F: Fn(&str) -> bool,
{
    let Ok(dir_iter) = fs::read_dir(".") else {
        return false;
    };
    for entry_res in dir_iter {
        let Ok(entry) = entry_res else {
            return false;
        };
        if let Ok(filename) = entry.file_name().into_string() {
            if f(&filename) {
                return true;
            }
        }
    }
    false
}

fn upfind(start: &Path, filename: &str) -> Option<PathBuf> {
    start
        .ancestors()
        .map(|path| path.join(filename))
        .find(file_exists)
}

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

fn find_git_root(workdir: &Path) -> Option<PathBuf> {
    Some(upfind(workdir, ".git")?.parent()?.to_path_buf())
}

fn get_hostname() -> String {
    let hostname = fs::read_to_string("/etc/hostname").unwrap_or_else(|_| String::from("<host>"));
    String::from(hostname.trim())
}

fn autojoin(vec: Vec<String>) -> String {
    vec.into_iter()
        .filter(|el| !el.is_empty())
        .collect::<Vec<String>>()
        .join(" ")
}

fn is_self_root() -> bool {
    getuid().is_root()
}

struct StatusLine {
    hostname: String,
    read_only: bool,
    git_root: Option<PathBuf>,
    current_home: Option<(PathBuf, String)>,
    build_info: String,
    workdir: PathBuf,
    username: String,
}

impl StatusLine {
    fn new() -> Self {
        let username = env::var("USER").unwrap_or_else(|_| String::from("<user>"));
        let workdir = env::current_dir().unwrap_or_else(|_| PathBuf::new());
        let read_only = access(&workdir, AccessFlags::W_OK).is_err();
        StatusLine {
            hostname: get_hostname(),
            read_only,
            git_root: find_git_root(&workdir),
            current_home: find_current_home(&workdir, &username),
            build_info: buildinfo(&workdir),
            workdir,
            username,
        }
    }

    fn get_workdir_str(&self) -> String {
        let (middle, highlighted) = match (&self.git_root, &self.current_home) {
            (Some(git_root), Some((home_root, _))) => {
                if home_root.starts_with(git_root) {
                    (None, self.workdir.strip_prefix(home_root).ok())
                } else {
                    (
                        git_root.strip_prefix(home_root).ok(),
                        self.workdir.strip_prefix(git_root).ok(),
                    )
                }
            }
            (Some(git_root), None) => (
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
            format!("/{}", String::from(middle.to_string_lossy()))
        } else {
            String::new()
        };

        let highlighted_str = if let Some(highlighted) = highlighted {
            let highlighted = highlighted.to_string_lossy();
            format!("{COLOR_CYAN}/{}{STYLE_RESET}", highlighted)
        } else {
            String::new()
        };

        format!("{}{}{}", home_str, middle_str, highlighted_str)
    }
}

impl fmt::Display for StatusLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let host_str = format!(
            "{STYLE_BOLD}{COLOR_YELLOW}(at {}{STYLE_BOLD}{COLOR_YELLOW}){STYLE_RESET}",
            colorize(&self.hostname)
        );
        let user_str = format!(
            "{STYLE_BOLD}{COLOR_BLUE}[as {}{STYLE_BOLD}{COLOR_BLUE}]{STYLE_RESET}",
            colorize(&self.username)
        );
        let hostuser = format!("{host_str} {user_str}");

        let workdir_str = self.get_workdir_str();
        let read_only_str = if self.read_only {
            concatcp!(COLOR_RED, "R/O", STYLE_RESET, " ")
        } else {
            ""
        };
        let pwd = format!("{read_only_str}{workdir_str}");

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
            if is_self_root() {
                concatcp!(COLOR_RED, "#")
            } else {
                concatcp!(COLOR_GREEN, "$")
            },
        );

        let top_left_line = format!("{}", autojoin(vec![hostuser, buildinfo, pwd]));
        let top_line = top_left_line; // TODO add time
        let top_line = format!(
            "{}{}{}",
            concatcp!(INVISIBLE_START, CURSOR_SAVE, CURSOR_UP, CURSOR_HOME),
            top_line,
            concatcp!(CURSOR_RESTORE, INVISIBLE_END)
        );

        let bottom_line = autojoin(vec![root_str]); // TODO add jobs and retval

        write!(f, "\n\n{}{} ", top_line, bottom_line)
    }
}

fn main() {
    print!("{}", StatusLine::new());
}
