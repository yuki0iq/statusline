use statusline::StatusLine;
use std::{env, fs};

fn main() {
    let exec = fs::read_link("/proc/self/exe")
        .map(|pb| String::from(pb.to_string_lossy()))
        .unwrap_or("<executable>".to_owned());
    let mut args = env::args();
    args.next();
    match args.next().as_deref() {
        Some("--env") => {
            println!(include_str!("shell.sh").replace("<exec>", exec));
        }
        Some("--run") => {
            let args = args.collect::<Vec<String>>();
            print!("{}", StatusLine::from_env(&args));
        }
        _ => {
            println!("Bash status line --- written in rust. Add `eval \"$(\"{exec}\" --env)\"` to your .bashrc!");
        }
    }
}
