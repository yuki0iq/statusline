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
            println!(
                "{}\nexport PS1='$(\"{exec}\" --run \"$?\" \"\\j\" \"$PS1_ELAPSED\")'",
                [
                    "export PS1_START=\"${EPOCHREALTIME/,/}\"",
                    "get_elapsed_time() {",
                    "    if [[ -n \"$PS1_START\" ]]; then",
                    "        PS1_END=\"${EPOCHREALTIME/,/}\"",
                    "        PS1_ELAPSED=\"$((PS1_END - PS1_START))\"",
                    "        PS1_START=",
                    "    else",
                    "        PS1_ELAPSED=0",
                    "    fi",
                    "}",
                    "export PS0='${PS1_START:0:$((PS1_START=${EPOCHREALTIME/,/},0))}'",
                    "export PROMPT_COMMAND='get_elapsed_time'",
                ]
                .join("\n"),
            );
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
