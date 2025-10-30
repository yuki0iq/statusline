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
    clippy::enum_glob_use
)]

mod args;
mod block;
mod chassis;
mod default;
mod file;
mod icon;
mod style;
mod time;
mod virt;
mod workgroup;

use crate::{
    args::Environment,
    block::{Extend, Kind as BlockType},
    chassis::Chassis,
    icon::{Icon, IconMode, Pretty},
    style::Style,
};

use crate::workgroup::{SshChain, WorkgroupKey};
use argh::FromArgs;
use rustix::{
    fd::{AsRawFd as _, FromRawFd as _, OwnedFd},
    fs::{Mode, OFlags},
};
use std::{io::Write as _, path::PathBuf};
use style::horizontal_absolute;
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
    #[argh(switch, hidden_help)]
    env: bool,

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
    jobs_count: usize,

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

impl From<Run> for Environment {
    fn from(other: Run) -> Environment {
        let ret_code = other.return_code;
        let jobs_count = other.jobs_count;
        let elapsed_time = other.elapsed_time;

        let work_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from(std::env::var("PWD").unwrap()));

        let git_tree = file::upfind(&work_dir, ".git").map(|dg| dg.parent().unwrap().to_path_buf());

        // XXX: This does not work well under Termux
        let user = std::env::var("USER")
            .unwrap_or_else(|_| format!("<user{}>", rustix::process::getuid().as_raw()));
        let host = rustix::system::uname()
            .nodename()
            .to_string_lossy()
            .into_owned();
        let chassis = Chassis::get();

        let current_home = file::find_current_home(&work_dir, &user);

        let mode = match other.mode.as_deref() {
            Some("text") => IconMode::Text,
            Some("minimal") => IconMode::MinimalIcons,
            _ => IconMode::Icons,
        };

        Environment {
            ret_code,
            jobs_count,
            elapsed_time,
            work_dir,
            git_tree,
            user,
            host,
            chassis,
            current_home,
            mode,
        }
    }
}

fn main() {
    let exec = std::fs::read_link("/proc/self/exe")
        .map(|pb| String::from(pb.to_string_lossy()))
        .unwrap_or("<executable>".to_owned());

    let mut args: Arguments = argh::from_env();
    if args.env {
        eprintln!(
            "{}",
            "`statusline --env` is deprecated."
                .visible()
                .yellow()
                .with_reset()
                .invisible()
        );
        eprintln!("Please replace it with `statusline env` in your ~/.bashrc");
        eprintln!("------------");
        args = Arguments {
            env: false,
            command: Some(Command::Env(Env {})),
        };
    }

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
        Command::Colorize(Colorize { what }) => println!("{}", what.colorize_with(&what).bold()),
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
        Command::Run(run) => print_statusline(run),
    }
}

fn print_statusline(run: Run) {
    if let Some(fd) = run.control_fd {
        // SAFETY: This file descriptor is already open
        let controlling_fd = unsafe { OwnedFd::from_raw_fd(fd) };
        // SAFETY: This file descriptor is not reused for concurrently running invocations.
        unsafe {
            libc::fcntl(
                controlling_fd.as_raw_fd(),
                libc::F_SETOWN,
                rustix::process::getpid(),
            )
        };
        rustix::fs::fcntl_setfl(controlling_fd, OFlags::ASYNC).unwrap();
    }

    let environ: Environment = run.into();
    let mode = environ.mode;

    let title = default::title(&environ);

    let bottom = default::pretty(&default::bottom(&environ), mode);

    let right = default::pretty(&default::right(&environ), mode);

    let workdir = BlockType::Workdir
        .create_from_env(&environ)
        .pretty(mode)
        .unwrap();
    let cont = BlockType::Continue
        .create_from_env(&environ)
        .pretty(mode)
        .unwrap();

    let line = default::top(&environ);

    let terminal_width: usize = terminal_size::terminal_size()
        .map_or(80, |(w, _h)| w.0)
        .into();

    let right_length = readline_width(&right);
    let right_formatted = format!(
        "{}{right}",
        // XXX: This may not be the right way to set right prompt...
        horizontal_absolute(terminal_width.saturating_sub(right_length))
    );

    let line_formatted = default::pretty(&line, mode);

    let three_line_mode =
        readline_width(&line_formatted) + readline_width(&workdir) + right_length + 16
            >= terminal_width;

    let prologue = crate::style::prologue(three_line_mode);
    let epilogue = crate::style::epilogue();

    let eprint_top_part = |top| {
        if three_line_mode {
            eprint!("{prologue}{top}{right_formatted}\n{cont} {workdir}{epilogue}");
        } else {
            eprint!("{prologue}{top} {workdir}{right_formatted}{epilogue}");
        }
    };

    eprint!("{title}");
    if three_line_mode {
        eprint!("\n\n\n");
    } else {
        eprint!("\n\n");
    }
    eprint_top_part(line_formatted);

    print!("{bottom}");
    std::io::stdout().flush().unwrap();
    rustix::stdio::dup2_stdout(rustix::fs::open("/dev/null", OFlags::RDWR, Mode::empty()).unwrap())
        .unwrap();

    let line = default::extend(line);
    eprint_top_part(default::pretty(&line, mode));
}
