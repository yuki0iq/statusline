// #![feature(adt_const_params)]
use const_format::{concatcp, formatcp};
use std::{env, fs};

const INVISIBLE_START: &str = "\x01";
const INVISIBLE_END: &str = "\x02";
const ESC: &str = "\x1b";
const BEL: &str = "\x07";

macro_rules! invisible {
    ( $x: expr ) => {
        concatcp!(INVISIBLE_START, $x, INVISIBLE_END)
    };
}

macro_rules! esc {
    ( $x: expr ) => {
        concatcp!(ESC, $x)
    };
}

const CSI: &str = esc!("[");

macro_rules! csi {
    ( $x: expr ) => {
        invisible!(concatcp!(CSI, $x))
    };
}

macro_rules! style {
    ( $x: expr ) => {
        csi!(concatcp!($x, "m"))
    };
}

macro_rules! color {
    ( $x: expr ) => {
        style!($x)
    };
}

macro_rules! rgb {
    ( $r: expr, $g: expr, $b: expr ) => {
        style!(formatcp!("38;2;{};{};{}", $r, $g, $b))
    };
}

// TODO compile-time
fn rgb(r: u8, g: u8, b: u8) -> String {
    format!("{INVISIBLE_START}{CSI}38;2;{r};{g};{b}m{INVISIBLE_END}")
}


const STYLE_RESET: &str = color!(0u8);
const STYLE_BOLD: &str = color!(1u8);
const COLOR_RED: &str = color!(31u8);
const COLOR_GREEN: &str = color!(32u8);
const COLOR_YELLOW: &str = color!(33u8);
const COLOR_PURPLE: &str = color!(35u8);
const COLOR_CYAN: &str = color!(36u8);
const COLOR_BLUE: &str = rgb!(0u8, 127u8, 240u8);
const COLOR_PINK: &str = rgb!(255u8, 100u8, 203u8);
const COLOR_PY_YELLOW: &str = rgb!(255u8, 219u8, 59u8);
const COLOR_LIGHT_GREEN: &str = rgb!(100u8, 255u8, 100u8);
const COLOR_LIGHT_RED: &str = rgb!(255u8, 80u8, 100u8);
const COLOR_GREY: &str = rgb!(128u8, 128u8, 128u8);

macro_rules! title {
    ( $x: expr ) => {
        invisible!(concatcp!(ESC, "]0;", $x, BEL))
    };
}

const CURSOR_SAVE: &str = esc!("[s");
const CURSOR_RESTORE: &str = esc!("[u");
const CURSOR_UP: &str = esc!("[A");
const CURSOR_HOME: &str = esc!("[G");

/*
How to "colorize" a string

using colors table
   F   C   A   6   0
 255 203 153 100   0
 -> scientific "pick" moment

allow (block too dark and too red)
FCA60 FCA6 FCA60
FCA60 0 FCA

block (r06 r00)

total 115 of 5^3 = 125, ban 10 highest
store in BGR (ban 00r, 06r)

then for hash from 0 to 114
B = hash / 25
G = hash / 5 % 5
R = hash % 5

COLOR_TABLE=(255 203 153 100 0)
*/
const COLOR_TABLE: [u8; 5] = [255, 203, 153, 100, 0];
fn colorize(s: &str) -> String {
    let hash = usize::from_str_radix(&sha256::digest(format!("{}\n", s))[..4], 16).unwrap() % 115;
    // println!("{}", &sha256::digest(format!("{}\n", s))[..4]);
    println!("{}", hash);
    let b = COLOR_TABLE[hash / 25];
    let g = COLOR_TABLE[hash / 5 % 5];
    let r = COLOR_TABLE[hash % 5];
    format!("{}{s}{STYLE_RESET}", rgb(r, g, b))
}

fn main() {
    let hostname = fs::read_to_string("/etc/hostname").unwrap_or(String::from("<host>"));
    let hostname = hostname.trim();
    let username = env::var("USER").unwrap_or(String::from("<user>"));
    let workdir = env::var("PWD").unwrap_or(String::new());

    println!("{STYLE_BOLD}{COLOR_YELLOW}(at {}{STYLE_BOLD}{COLOR_YELLOW}){STYLE_RESET} {STYLE_BOLD}{COLOR_BLUE}[as {}{STYLE_BOLD}{COLOR_BLUE}]{STYLE_RESET} -> {}", colorize(&hostname), colorize(&username), colorize(&workdir));
}
