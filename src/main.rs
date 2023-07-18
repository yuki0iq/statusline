#![feature(adt_const_params)]

use const_format::{concatcp, formatcp};

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

fn main() {
    println!("{COLOR_YELLOW}Hello, world!{STYLE_RESET}");
}
