#![warn(
    clippy::cargo,
    clippy::pedantic,
    clippy::allow_attributes,
    clippy::as_underscore,
    clippy::assertions_on_result_states,
    clippy::cfg_not_test,
    clippy::clone_on_ref_ptr,
    clippy::create_dir,
    clippy::decimal_literal_representation,
    clippy::default_numeric_fallback,
    clippy::default_union_representation,
    clippy::deref_by_slicing,
    clippy::empty_drop,
    clippy::empty_enum_variants_with_brackets,
    clippy::exhaustive_enums,
    clippy::fn_to_numeric_cast_any,
    clippy::format_push_string,
    clippy::get_unwrap,
    clippy::host_endian_bytes,
    clippy::if_then_some_else_none,
    clippy::infinite_loop,
    clippy::inline_asm_x86_att_syntax,
    clippy::let_underscore_must_use,
    clippy::lossy_float_literal,
    clippy::map_err_ignore,
    clippy::mem_forget,
    clippy::missing_assert_message,
    clippy::mixed_read_write_in_expression,
    clippy::multiple_unsafe_ops_per_block,
    clippy::mutex_atomic,
    clippy::needless_raw_strings,
    clippy::partial_pub_fields,
    clippy::pathbuf_init_then_push,
    clippy::rc_buffer,
    clippy::rc_mutex,
    clippy::redundant_type_annotations,
    clippy::renamed_function_params,
    clippy::semicolon_outside_block,
    clippy::shadow_same,
    clippy::shadow_unrelated,
    clippy::str_to_string,
    clippy::string_lit_chars_any,
    clippy::suspicious_xor_used_as_pow,
    clippy::tests_outside_test_module,
    clippy::try_err,
    clippy::undocumented_unsafe_blocks,
    clippy::unnecessary_safety_comment,
    clippy::unnecessary_safety_doc,
    clippy::unneeded_field_pattern,
    clippy::unseparated_literal_suffix,
    clippy::unused_result_ok,
    clippy::unused_trait_names,
    clippy::verbose_file_reads
)]
#![allow(
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::multiple_crate_versions,
    clippy::case_sensitive_file_extension_comparisons,
    clippy::enum_glob_use,
    clippy::unnecessary_box_returns,
    clippy::wildcard_imports,
    clippy::new_ret_no_self
)]

mod block;
mod chassis;
mod file;
mod icon;
mod style;
mod virt;
mod workgroup;

use crate::{
    block::{Block, create_blocks},
    chassis::Chassis,
    icon::{Icon, IconMode, Pretty},
    style::{Color, Style, WithStyle, horizontal_absolute},
    workgroup::{SshChain, WorkgroupKey},
};
use argh::FromArgs;
use pwd::Passwd;
use rustix::{
    fd::{FromRawFd as _, OwnedFd},
    fs::{Mode, OFlags},
};
use std::{fmt::Write as _, io::Write as _, os::fd::AsRawFd as _, path::PathBuf, time::Duration};
use unicode_width::UnicodeWidthStr as _;

fn readline_width(s: &str) -> usize {
    let mut res = s.width();
    for (i, c) in s.bytes().enumerate() {
        match c {
            b'\x01' => res += i,
            b'\x02' => res -= i + 1,
            _ => {}
        }
    }
    res
}

#[derive(FromArgs)]
/// statusline
struct Arguments {
    #[argh(subcommand)]
    /// action
    command: Option<Command>,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum Command {
    Colorize(Colorize),
    WorkgroupCreate(WorkgroupCreate),
    Chain(Chain),
    Run(Run),
    Env(Env),
}

#[derive(FromArgs)]
#[argh(subcommand, name = "env")]
/// print bash commands
struct Env {}

#[derive(FromArgs)]
#[argh(subcommand, name = "chain")]
/// append this host to chain
struct Chain {}

#[derive(FromArgs)]
#[argh(subcommand, name = "create")]
/// create for this host
struct WorkgroupCreate {}

#[derive(FromArgs)]
#[argh(subcommand, name = "colorize")]
/// colorize as username
struct Colorize {
    #[argh(option)]
    /// what to colorize
    what: String,
}

#[derive(FromArgs)]
#[argh(subcommand, name = "run")]
/// main statusline
struct Run {
    #[argh(option)]
    /// return code to show
    return_code: Option<u8>,

    #[argh(option)]
    /// current background jobs count
    jobs_count: Option<usize>,

    #[argh(option)]
    /// elapsed time to show, in seconds
    elapsed_time: Option<u64>,

    #[argh(option)]
    /// control fd for terminating
    control_fd: Option<i32>,

    #[argh(option)]
    /// icon mode. `text` and `minimal` have special meaning
    mode: Option<String>,
}

/// Environment variables available to statusline
pub struct Environment {
    /// Last command's return code
    pub ret_code: Option<u8>,
    /// Jobs currently running
    pub jobs_count: Option<usize>,
    /// Last command's elapsed time, in us
    pub elapsed_time: Option<Duration>,
    /// Working directory
    pub work_dir: PathBuf,
    /// Git worktree path if any
    pub git_tree: Option<PathBuf>,
    /// Username
    pub user: String,
    /// Hostname
    pub host: String,
    /// Current home: dir and username
    pub current_home: Option<(PathBuf, String)>,
}

impl From<Run> for Environment {
    fn from(other: Run) -> Environment {
        let ret_code = other.return_code;
        let jobs_count = other.jobs_count;
        let elapsed_time = other.elapsed_time.map(Duration::from_micros);

        let work_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from(std::env::var("PWD").unwrap()));

        let git_tree = file::upfind(&work_dir, ".git").map(|dg| dg.parent().unwrap().to_path_buf());

        // XXX: This probably does not work well under Termux
        let user = Passwd::current_user()
            .map(|entry| entry.name)
            .or_else(|| std::env::var("USER").ok())
            .unwrap_or_else(|| format!("<user{}>", rustix::process::getuid().as_raw()));
        let host = rustix::system::uname()
            .nodename()
            .to_string_lossy()
            .into_owned();

        let current_home = file::find_current_home(&work_dir, &user);

        Environment {
            ret_code,
            jobs_count,
            elapsed_time,
            work_dir,
            git_tree,
            user,
            host,
            current_home,
        }
    }
}

fn main() {
    let exec = std::fs::read_link("/proc/self/exe")
        .map(|pb| pb.to_string_lossy().into_owned())
        .unwrap_or("<executable>".to_owned());

    let args: Arguments = argh::from_env();

    let Some(command) = args.command else {
        let ver = env!("CARGO_PKG_VERSION");
        let apply_me = format!("source <(\"{exec}\" env)");
        println!("[statusline {ver}] --- bash status line, written in Rust");
        println!(">> https://codeberg.org/yuki0iq/statusline");
        println!("Use `--help` to see advanced usage");
        println!("Simple install:");
        println!("    echo '{apply_me}' >> ~/.bashrc");
        println!("    source ~/.bashrc");
        println!("Test now:");
        println!("    {apply_me}");
        return;
    };

    match command {
        Command::Colorize(Colorize { what }) => {
            let mut res = String::new();
            res.with_style(Color::of(&what), Style::BOLD, |f| write!(f, "{what}"))
                .unwrap();
            println!("{res}");
        }
        Command::WorkgroupCreate(_) => {
            WorkgroupKey::create().expect("Could not create workgroup key");
        }
        Command::Env(_) => println!("{}", include_str!("shell.sh").replace("<exec>", &exec)),
        Command::Chain(_) => {
            let Ok(key) = WorkgroupKey::load() else {
                return;
            };
            let mut ssh_chain = SshChain::open(Some(&key)).0;
            ssh_chain.push(
                rustix::system::uname()
                    .nodename()
                    .to_string_lossy()
                    .into_owned(),
            );
            println!("{}", SshChain(ssh_chain).seal(&key));
        }
        Command::Run(run) => run_statusline(run),
    }
}

fn run_statusline(run: Run) {
    if let Some(fd) = run.control_fd {
        // SAFETY: This file descriptor is already open
        let fd = unsafe { OwnedFd::from_raw_fd(fd) };
        // SAFETY: This file descriptor is not reused for concurrently running invocations.
        unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_SETOWN, rustix::process::getpid()) };
        rustix::fs::fcntl_setfl(fd, OFlags::ASYNC).unwrap();
    }

    let mode = match run.mode.as_deref() {
        Some("text") => IconMode::Text,
        Some("minimal") => IconMode::MinimalIcons,
        _ => IconMode::Icons,
    };

    let environ: Environment = run.into();

    let bottom = create_blocks(&["root_shell"], &environ);

    let right = create_blocks(&["elapsed", "return_code", "time"], &environ);

    let middle = create_blocks(&["workdir"], &environ);

    let mut left = create_blocks(
        &[
            "host_user",
            "ssh",
            "git_repo",
            "git_tree",
            "build_info",
            "nix_shell",
            "venv",
            "jobs",
            "unseen_mail",
        ],
        &environ,
    );

    print_statusline(mode, &environ, &mut left, &middle, &right, &bottom);
}

fn print_statusline(
    mode: IconMode,
    environ: &Environment,
    left: &mut [Box<dyn Block>],
    middle: &[Box<dyn Block>],
    right: &[Box<dyn Block>],
    bottom: &[Box<dyn Block>],
) {
    let middle = pretty(middle, mode);
    let right = pretty(right, mode);
    let bottom = pretty(bottom, mode);

    let cont = if let IconMode::Text = mode {
        ">"
    } else {
        "\u{f105}"
    };

    let terminal_width: usize = terminal_size::terminal_size()
        .map_or(80, |(w, _h)| w.0)
        .into();

    let right_length = readline_width(&right);
    let right_formatted = format!(
        "{}{right}",
        // XXX: This may not be the right way to set right prompt...
        horizontal_absolute(terminal_width.saturating_sub(right_length))
    );

    let left_formatted = pretty(left, mode);

    let three_line_mode =
        readline_width(&left_formatted) + readline_width(&middle) + right_length + 16
            >= terminal_width;

    let prologue = crate::style::prologue(three_line_mode);
    let epilogue = crate::style::epilogue();

    let eprint_top_part = |top| {
        if three_line_mode {
            eprint!("{prologue}{top}{right_formatted}\n{cont} {middle}{epilogue}");
        } else {
            eprint!("{prologue}{top} {middle}{right_formatted}{epilogue}");
        }
    };

    let title = make_title(environ);
    eprint!("{title}");

    if three_line_mode {
        eprint!("\n\n\n");
    } else {
        eprint!("\n\n");
    }
    eprint_top_part(left_formatted);

    print!("{bottom} ");
    std::io::stdout().flush().unwrap();
    rustix::stdio::dup2_stdout(rustix::fs::open("/dev/null", OFlags::RDWR, Mode::empty()).unwrap())
        .unwrap();

    for block in &mut *left {
        block.extend();
    }
    eprint_top_part(pretty(left, mode));
}

fn make_title(env: &Environment) -> String {
    let pwd = if let Some((home, user)) = &env.current_home {
        let wd = env.work_dir.strip_prefix(home).unwrap_or(&env.work_dir);
        if wd.as_os_str().is_empty() {
            format_args!("~{}", *user)
        } else {
            format_args!("~{}/{}", *user, wd.display())
        }
    } else {
        format_args!("{}", env.work_dir.display())
    };
    crate::style::title(&format!("{}@{}: {}", env.user, env.host, pwd))
}

fn pretty(line: &[Box<dyn Block>], mode: IconMode) -> String {
    let mut res = String::new();
    for block in line {
        let prev_len = res.len();
        write!(res, "{}", crate::icon::display(block.as_ref(), mode)).unwrap();
        if prev_len != res.len() {
            res.push(' ');
        }
    }
    res.pop();
    res
}
