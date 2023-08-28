use libc::fcntl as fcntl_unsafe;
use nix::{
    fcntl::{self, FcntlArg, OFlag},
    unistd,
};
use statusline::{default, Environment, Icons, Pretty, Style, Top};
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

            let icons = Icons::build();
            let args = Environment::from_env(&args.collect::<Vec<String>>());
            let line = Top::from(&args);
            let bottom = default::bottom(&args);

            let top_line = |line: &Top| {
                line.pretty(&icons)
                    .unwrap_or_default()
                    .prev_line(1)
                    .save_restore()
                    .to_string()
            };

            eprint!("{}", top_line(&line));

            print!(
                "{}{}",
                default::title(&args).invisible(),
                bottom.as_slice().pretty(&icons).unwrap_or_default()
            );
            io::stdout().flush().unwrap();
            unistd::close(1).unwrap();

            let line = line.extended();
            eprint!("{}", top_line(&line));
        }
        _ => {
            let ver = env!("CARGO_PKG_VERSION");
            println!("[statusline {ver}] --- bash status line, written in Rust");
            println!("Simple install:");
            println!("    eval \"$(\"{exec}\" --env)\" >> ~/.bashrc");
        }
    }
}
