use const_format::{concatcp, formatcp};
use sha256::digest;

pub const INVISIBLE_START: &str = "\x01";
pub const INVISIBLE_END: &str = "\x02";
pub const ESC: &str = "\x1b";
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

fn rgb(r: u8, g: u8, b: u8) -> String {
    format!("{INVISIBLE_START}{CSI}38;2;{r};{g};{b}m{INVISIBLE_END}")
}

pub const STYLE_RESET: &str = color!(0u8);
pub const STYLE_BOLD: &str = color!(1u8);

pub const COLOR_RED: &str = color!(31u8);
pub const COLOR_GREEN: &str = color!(32u8);
pub const COLOR_YELLOW: &str = color!(33u8);
pub const COLOR_PURPLE: &str = color!(35u8);
pub const COLOR_CYAN: &str = color!(36u8);
pub const COLOR_BLUE: &str = rgb!(0u8, 127u8, 240u8);
pub const COLOR_PINK: &str = rgb!(255u8, 100u8, 203u8);
//pub const COLOR_PY_YELLOW: &str = rgb!(255u8, 219u8, 59u8);
pub const COLOR_LIGHT_GREEN: &str = rgb!(100u8, 255u8, 100u8);
pub const COLOR_LIGHT_RED: &str = rgb!(255u8, 80u8, 100u8);
pub const COLOR_GREY: &str = rgb!(128u8, 128u8, 128u8);

pub fn title(s: &str) -> String {
    format!("{INVISIBLE_START}{ESC}]0;{}{BEL}{INVISIBLE_END}", s)
}

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
pub fn colorize(s: &str) -> String {
    if s == "root" {
        String::from(concatcp!(STYLE_BOLD, COLOR_RED, "root", STYLE_RESET))
    } else {
        let hash = usize::from_str_radix(&digest(format!("{}\n", s))[..4], 16).unwrap() % 115;
        let b = COLOR_TABLE[hash / 25];
        let g = COLOR_TABLE[hash / 5 % 5];
        let r = COLOR_TABLE[hash % 5];
        format!("{}{s}{STYLE_RESET}", rgb(r, g, b))
    }
}
