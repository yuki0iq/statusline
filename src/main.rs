use libc::{fcntl as fcntl_unsafe, F_SETOWN};
use nix::{fcntl, unistd};
use statusline::StatusLine;
use std::{env, fs, io, io::Write};

fn main() {
    let exec = fs::read_link("/proc/self/exe")
        .map(|pb| String::from(pb.to_string_lossy()))
        .unwrap_or("<executable>".to_owned());
    let mut args = env::args();
    args.next();
    match args.next().as_deref() {
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

            print!("{}", line);
            io::stdout().flush().unwrap();
            unistd::close(1).unwrap();

            // thread::sleep(time::Duration::from_secs(10));
            let line = line.extended();
            // print!("{line}");
            eprint!("\x1b[s\x1b[G\x1b[2A{}\x1b[u", line);
        }
        _ => {
            println!("Bash status line --- written in rust. Add `eval \"$(\"{exec}\" --env)\"` to your .bashrc!");
        }
    }
}
