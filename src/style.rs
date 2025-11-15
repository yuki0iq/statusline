use bitflags::bitflags;
use const_format::formatcp;
use std::fmt::{Display, Formatter, Result as FmtResult, Write};

const INVISIBLE_START: &str = "\x01";
const INVISIBLE_END: &str = "\x02";
const BEL: &str = "\x07";
const ESC: &str = "\x1b";
const CSI: &str = formatcp!("{ESC}[");
const OSC: &str = formatcp!("{ESC}]");
const RESET: &str = formatcp!("{CSI}0m");
const CURSOR_SAVE: &str = formatcp!("{CSI}s");
const CURSOR_RESTORE: &str = formatcp!("{CSI}u");
const CLEAR_LINE: &str = formatcp!("{CSI}0K");

fn prev_line(n: usize) -> String {
    format!("{CSI}{n}A{CSI}G")
}

pub fn prologue(three_line_mode: bool) -> String {
    format!(
        "{INVISIBLE_START}{CURSOR_SAVE}{}{CLEAR_LINE}{INVISIBLE_END}",
        prev_line(if three_line_mode { 2 } else { 1 })
    )
}
pub fn epilogue() -> String {
    format!("{INVISIBLE_START}{CURSOR_RESTORE}{INVISIBLE_END}")
}
pub fn title(title: &str) -> String {
    format!("{INVISIBLE_START}{OSC}0;{title}{BEL}{INVISIBLE_END}")
}
pub fn horizontal_absolute(n: usize) -> String {
    format!("{INVISIBLE_START}{CSI}{n}G{INVISIBLE_END}")
}

bitflags! {
    pub struct Style: u8 {
        const BOLD = 0x01;
        const ITALIC = 0x02;
    }
}

impl Display for Style {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if self.contains(Self::BOLD) {
            write!(f, "{CSI}1m")?;
        }
        if self.contains(Self::ITALIC) {
            write!(f, "{CSI}3m")?;
        }
        Ok(())
    }
}

pub enum Color {
    // CSI 30 + {}
    Low8(u8),
    // CSI 90 + {}
    High8(u8),
    True(u8, u8, u8),
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Low8(low) => write!(f, "{CSI}{}m", 30 + low),
            Self::High8(high) => write!(f, "{CSI}{}m", 90 + high),
            Self::True(r, g, b) => write!(f, "{CSI}38;2;{r};{g};{b}m"),
        }
    }
}

impl Color {
    pub const RED: Self = Self::Low8(1);
    pub const GREEN: Self = Self::Low8(2);
    pub const YELLOW: Self = Self::Low8(3);
    // pub const BLUE: Self = Self::Low8(4);
    pub const PURPLE: Self = Self::Low8(5);
    pub const CYAN: Self = Self::Low8(6);
    pub const LIGHT_GRAY: Self = Self::Low8(7);

    pub const BRIGHT_BLUE: Self = Self::High8(4);

    pub const PINK: Self = Self::True(255, 100, 203);
    pub const LIGHT_GREEN: Self = Self::True(100, 255, 100);
    pub const LIGHT_RED: Self = Self::True(255, 80, 100);
    pub const GRAY: Self = Self::True(128, 128, 128);
    pub const TRUE_YELLOW: Self = Self::True(255, 170, 0);

    pub fn of(what: &str) -> Self {
        fn polyhash(s: &str, m: usize, p: usize, h_init: usize) -> usize {
            let mut h = h_init % m;
            for by in s.bytes() {
                h = (h * p + by as usize) % m;
            }
            h
        }

        if what == "root" {
            Self::RED
        } else {
            let idx = polyhash(what, 23, 179, what.len()) + 1;
            let (r, g, b) = HSV_COLOR_TABLE[idx];
            Self::True(r, g, b)
        }
    }
}

const HSV_COLOR_TABLE: [(u8, u8, u8); 24] = [
    (255, 0, 0),
    (255, 85, 0),
    (255, 128, 0),
    (255, 170, 0),
    (255, 213, 0),
    (255, 255, 0),
    (213, 255, 0),
    (170, 255, 0),
    (128, 255, 0),
    (0, 255, 85),
    (0, 255, 128),
    (0, 255, 170),
    (0, 255, 213),
    (0, 213, 255),
    (0, 128, 255),
    (0, 85, 255),
    (128, 0, 255),
    (170, 0, 255),
    (213, 0, 255),
    (255, 0, 255),
    (255, 0, 212),
    (255, 0, 170),
    (255, 0, 128),
    (255, 0, 85),
];

pub trait WithStyle: Write {
    fn with_style<F>(&mut self, color: Color, style: Style, func: F) -> FmtResult
    where
        F: FnOnce(&mut Self) -> FmtResult,
    {
        write!(self, "{INVISIBLE_START}{color}{style}{INVISIBLE_END}")?;
        func(self)?;
        write!(self, "{INVISIBLE_START}{RESET}{INVISIBLE_END}")?;
        Ok(())
    }
}

impl<T: Write> WithStyle for T {}
