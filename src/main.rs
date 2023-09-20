use nix::{
    fcntl::{self, FcntlArg, OFlag},
    libc,
    unistd,
};
use statusline::{default, Environment, IconMode, Style};
use std::{env, fs};
use unicode_width::UnicodeWidthChar;

fn readline_width(s: &str) -> usize {
    let mut res = 0;
    let mut skip = false;
    for c in s.chars() {
        match c {
            '\x01' => skip = true,
            '\x02' => skip = false,
            c if !skip => res += c.width().unwrap_or(0),
            _ => {}
        }
    }
    res
}

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
        Some("--top") => {
            unsafe {
                libc::fcntl(3, libc::F_SETOWN, unistd::getpid());
            }
            fcntl::fcntl(3, FcntlArg::F_SETFL(OFlag::O_ASYNC)).unwrap();

            use statusline::BlockType; //<===

            let mode = IconMode::build();
            let args = Environment::from_env::<&str>(&[]);

            let mut line: [Box<_>; 8] = [
                BlockType::HostUser,
                BlockType::GitRepo,
                BlockType::GitTree,
                BlockType::BuildInfo,
                BlockType::Venv,
                BlockType::Workdir, //<===[5]
                BlockType::Elapsed,
                BlockType::Time,
            ]
            .map(|x| x.create_from_env(&args));

            let line_length: usize = line
                .iter()
                .map(|x| x.pretty(&mode))
                .filter_map(|x| x)
                .map(|x| readline_width(&x))
                .sum();

            let make_line = |line: String, up: i32| {
                line.clear_till_end()
                    .prev_line(up)
                    .save_restore()
                    .to_string()
            };

            if line_length + 25 >= term_size::dimensions().map(|s| s.0).unwrap_or(80) {
                // three lines
                print!("\n\n\n");

                let mut second = BlockType::Empty.create_from_env(&args);
                std::mem::swap(&mut second, &mut line[5]);
                let second = [BlockType::Continue.create_from_env(&args), second];

                eprint!(
                    "{}{}",
                    make_line(default::pretty(&line, &mode), 2),
                    make_line(default::pretty(&second, &mode), 1)
                );

                let line = default::extend(line);
                let second = default::extend(second);
                eprint!(
                    "{}{}",
                    make_line(default::pretty(&line, &mode), 2),
                    make_line(default::pretty(&second, &mode), 1)
                );
            } else {
                // two lines
                print!("\n\n");

                eprint!("{}", make_line(default::pretty(&line, &mode), 1));

                let line = default::extend(line);
                eprint!("{}", make_line(default::pretty(&line, &mode), 1));
            }
        }
        Some("--bottom") => {
            let mode = IconMode::build();
            let args = Environment::from_env(&args.collect::<Vec<String>>());
            let bottom = default::bottom(&args);

            print!(
                "{}{}",
                default::title(&args).invisible(),
                default::pretty(&bottom, &mode)
            );
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
