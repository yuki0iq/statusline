use libc::{fcntl as fcntl_unsafe, F_SETOWN};
use nix::{fcntl, unistd};
use statusline::{
    style::{colorize, INVISIBLE_END, INVISIBLE_START},
    StatusLine,
};
use std::{env, fs, io, io::Write};

fn main() {
    let exec = fs::read_link("/proc/self/exe")
        .map(|pb| String::from(pb.to_string_lossy()))
        .unwrap_or("<executable>".to_owned());
    let mut args = env::args();
    args.next();
    match args.next().as_deref() {
        Some("--colorize") => match args.next() {
            Some(text) => println!("{}", colorize(&text, &text)),
            None => println!("`statusline --colorize <text>` to colorize string"),
        },
        Some("--env") => {
            println!("{}", include_str!("shell.sh").replace("<exec>", &exec));
        }
        Some("--run") => {
            unsafe {
                fcntl_unsafe(0, F_SETOWN, unistd::getpid());
            }
            fcntl::fcntl(0, fcntl::FcntlArg::F_SETFL(fcntl::OFlag::O_ASYNC)).unwrap();

            let args = args.collect::<Vec<String>>();
            let line = StatusLine::from_env(&args);

            // TODO: fix "job terminated" and some other stderr-printed lines being overwritten
            eprint!(
                "\x1b[s\x1b[G\x1b[A{}\x1b[u",
                line.to_top()
                    .replace(INVISIBLE_START, "")
                    .replace(INVISIBLE_END, ""),
            );
            print!("{}{}", line.to_title(), line.to_bottom());
            io::stdout().flush().unwrap();
            unistd::close(1).unwrap();

            let line = line.extended();
            eprint!("\x1b[s\x1b[G\x1b[A{}\x1b[u", line.to_top());
        }
        _ => {
            println!("Bash status line --- written in rust. Add `eval \"$(\"{exec}\" --env)\"` to your .bashrc!");
        }
    }
}
