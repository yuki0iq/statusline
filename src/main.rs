use argh::FromArgs;
use rustix::{
    fd::{self, AsRawFd, FromRawFd},
    fs as rfs, process, stdio,
};
use statusline::{
    BlockType, Chassis, Environment, IconMode, Style, default, file,
    workgroup::{SshChain, WorkgroupKey},
};
use std::{env, fs, io, io::Write, path::PathBuf};
use unicode_width::UnicodeWidthStr;

fn readline_width(s: &str) -> usize {
    let mut res = s.width();
    for (i, c) in s.bytes().enumerate() {
        match c {
            b'\x01' => res += i + 1,
            b'\x02' => res -= i,
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
    /// control pid for terminating
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

        let work_dir =
            env::current_dir().unwrap_or_else(|_| PathBuf::from(env::var("PWD").unwrap()));

        let git_tree = file::upfind(&work_dir, ".git")
            .ok()
            .map(|dg| dg.parent().unwrap().to_path_buf());

        let user = env::var("USER").unwrap_or_else(|_| String::from("<user>"));
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
            git_tree,
            work_dir,
            user,
            host,
            chassis,
            current_home,
            mode,
        }
    }
}

fn main() {
    let exec = fs::read_link("/proc/self/exe")
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
        let apply_me = format!("eval \"$(\"{exec}\" env)\"");
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
            WorkgroupKey::create().expect("Could not create workgroup key")
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
        Command::Run(run) => {
            if let Some(fd) = run.control_fd {
                let controlling_fd = unsafe { fd::OwnedFd::from_raw_fd(fd) };
                unsafe {
                    libc::fcntl(
                        controlling_fd.as_raw_fd(),
                        libc::F_SETOWN,
                        process::getpid(),
                    )
                };
                let _ = rfs::fcntl_setfl(controlling_fd, OFlags::ASYNC);
            }

            let args: Environment = run.into();
            let mode = args.mode;
            let bottom = default::bottom(&args);

            let mut line = default::top(&args);

            let line_length: usize = line
                .iter()
                .filter_map(|x| x.pretty(&mode))
                .map(|x| readline_width(&x))
                .sum();

            if line_length + 16
                >= terminal_size::terminal_size()
                    .map(|(w, _h)| w.0)
                    .unwrap_or(80)
                    .into()
            {
                // three lines
                let mut second = BlockType::Empty.create_from_env(&args);
                std::mem::swap(&mut second, &mut line[8]);
                let second = [BlockType::Continue.create_from_env(&args), second];

                eprint!(
                    "\n\n\n{}",
                    default::pretty(&line, &mode)
                        .join_lf(default::pretty(&second, &mode))
                        .clear_till_end()
                        .prev_line(2)
                        .save_restore()
                );

                print!(
                    "{}{}",
                    default::title(&args).invisible(),
                    default::pretty(&bottom, &mode)
                );
                io::stdout().flush().unwrap();
                stdio::dup2_stdout(
                    rfs::open("/dev/null", rfs::OFlags::RDWR, rfs::Mode::empty()).unwrap(),
                )
                .unwrap();

                let line = default::extend(line);
                let second = default::extend(second);
                eprint!(
                    "{}",
                    default::pretty(&line, &mode)
                        .join_lf(default::pretty(&second, &mode))
                        .clear_till_end()
                        .prev_line(2)
                        .save_restore()
                );
            } else {
                // two lines
                eprint!(
                    "\n\n{}",
                    default::pretty(&line, &mode)
                        .clear_till_end()
                        .prev_line(1)
                        .save_restore()
                );

                print!(
                    "{}{}",
                    default::title(&args).invisible(),
                    default::pretty(&bottom, &mode)
                );
                io::stdout().flush().unwrap();
                stdio::dup2_stdout(
                    rfs::open("/dev/null", rfs::OFlags::RDWR, rfs::Mode::empty()).unwrap(),
                )
                .unwrap();

                let line = default::extend(line);
                eprint!(
                    "{}",
                    default::pretty(&line, &mode)
                        .clear_till_end()
                        .prev_line(1)
                        .save_restore()
                );
            }
        }
    }
}
