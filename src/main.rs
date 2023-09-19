use libc::fcntl as fcntl_unsafe;
use nix::{
    fcntl::{self, FcntlArg, OFlag},
    unistd,
};
use statusline::{default, Environment, IconMode, Style};
use std::{
    env, fs,
    io::{self, Write},
};

fn main() {
    let exec = fs::read_link("/proc/self/exe")
        .map(|pb| String::from(pb.to_string_lossy()))
        .unwrap_or("<executable>".to_owned());
    let mut args = env::args();
    args.next();
    match args.next().as_deref() {
        Some("--colorize") => match args.next() {
            Some(text) => println!("{}", text.colorize_with(&text).bold()),
            None => println!("`statusline --colorize <text>` to colorize string"),
        },
        Some("--env") => {
            println!("{}", include_str!("shell.sh").replace("<exec>", &exec));
        }
        Some("--run") => {
            unsafe {
                fcntl_unsafe(3, libc::F_SETOWN, unistd::getpid());
            }
            fcntl::fcntl(3, FcntlArg::F_SETFL(OFlag::O_ASYNC)).unwrap();

            let mode = IconMode::build();
            let args = Environment::from_env(&args.collect::<Vec<String>>());
            let line = default::top(&args);
            let bottom = default::bottom(&args);

            let top_line = |line: String| {
                line.clear_till_end()
                    .prev_line(1)
                    .save_restore()
                    .to_string()
            };

            eprint!("{}", top_line(default::pretty(&line, &mode)));

            print!(
                "{}{}",
                default::title(&args).invisible(),
                default::pretty(&bottom, &mode)
            );
            io::stdout().flush().unwrap();
            unistd::close(1).unwrap();

            let line = default::extend(line);
            eprint!("{}", top_line(default::pretty(&line, &mode)));
        }
        _ => {
            let ver = env!("CARGO_PKG_VERSION");
            let apply_me = format!("eval \"$(\"{exec}\" --env)\"");
            println!("[statusline {ver}] --- bash status line, written in Rust");
            println!(">> https://git.yukii.keenetic.pro/yuki0iq/statusline");
            println!("Simple install:");
            println!("    echo '{apply_me}' >> ~/.bashrc");
            println!("    source ~/.bashrc");
            println!("Test now:");
            println!("    {apply_me}");
        }
    }
}
