use std::fmt::{Display, Formatter, Result as FmtResult};

pub const INVISIBLE_START: &str = "\x01";
pub const INVISIBLE_END: &str = "\x02";
pub const ESC: &str = "\x1b";
const CSI: &str = "\x1b[";
const RESET: &str = "\x1b[0m";
const BEL: &str = "\x07";
const COLOR_TABLE: [u8; 5] = [255, 203, 153, 100, 0];

pub enum StyleKind {
    Title,
    Bold,
    Color8(usize),
    TrueColor(u8, u8, u8),
    ResetEnd,
    Invisible,
    Boxed,
    Rounded,
    CursorHorizontalAbsolute(i32),
    CursorPreviousLine(i32),
    CursorSaveRestore,
}

pub struct Styled<'a, T: Display + ?Sized> {
    style: StyleKind,
    value: &'a T,
}

impl<T: Display + ?Sized> Display for Styled<'_, T> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self.style {
            StyleKind::Title => write!(f, "{ESC}]0;{}{BEL}", self.value),
            StyleKind::Bold => write!(f, "{CSI}1m{}", self.value),
            StyleKind::Color8(index) => write!(f, "{CSI}{}m{}", index + 31, self.value),
            StyleKind::TrueColor(r, g, b) => {
                write!(f, "{CSI}38;2;{r};{g};{b}m{}", self.value)
            }
            StyleKind::ResetEnd => write!(f, "{}{RESET}", self.value),
            StyleKind::Invisible => write!(f, "{INVISIBLE_START}{}{INVISIBLE_END}", self.value),
            StyleKind::Boxed => write!(f, "[{}]", self.value),
            StyleKind::Rounded => write!(f, "({})", self.value),
            StyleKind::CursorHorizontalAbsolute(n) => write!(f, "{CSI}{n}G{}", self.value),
            StyleKind::CursorPreviousLine(n) => write!(f, "{CSI}{n}F{}", self.value),
            StyleKind::CursorSaveRestore => write!(f, "{CSI}s{}{CSI}u", self.value),
        }
    }
}

pub trait Style: Display {
    fn as_title(&self) -> Styled<Self> {
        Styled {
            style: StyleKind::Title,
            value: self,
        }
    }

    fn bold(&self) -> Styled<Self> {
        Styled {
            style: StyleKind::Bold,
            value: self,
        }
    }

    fn low_color(&self, index: usize) -> Styled<Self> {
        Styled {
            style: StyleKind::Color8(index),
            value: self,
        }
    }

    fn true_color(&self, red: u8, green: u8, blue: u8) -> Styled<Self> {
        Styled {
            style: StyleKind::TrueColor(red, green, blue),
            value: self,
        }
    }

    fn invisible(&self) -> Styled<Self> {
        Styled {
            style: StyleKind::Invisible,
            value: self,
        }
    }

    fn with_reset(&self) -> Styled<Self> {
        Styled {
            style: StyleKind::ResetEnd,
            value: self,
        }
    }

    fn boxed(&self) -> Styled<Self> {
        Styled {
            style: StyleKind::Boxed,
            value: self,
        }
    }

    fn rounded(&self) -> Styled<Self> {
        Styled {
            style: StyleKind::Rounded,
            value: self,
        }
    }

    fn horizontal_absolute(&self, pos: i32) -> Styled<Self> {
        Styled {
            style: StyleKind::CursorHorizontalAbsolute(pos),
            value: self,
        }
    }

    fn prev_line(&self, count: i32) -> Styled<Self> {
        Styled {
            style: StyleKind::CursorPreviousLine(count),
            value: self, 
        }
    }

    fn save_restore(&self) -> Styled<Self> {
        Styled {
            style: StyleKind::CursorSaveRestore,
            value: self
        }
    }

    fn red(&self) -> Styled<Self> {
        self.low_color(0)
    }

    fn green(&self) -> Styled<Self> {
        self.low_color(1)
    }

    fn yellow(&self) -> Styled<Self> {
        self.low_color(2)
    }

    fn blue(&self) -> Styled<Self> {
        self.low_color(3)
    }

    fn purple(&self) -> Styled<Self> {
        self.low_color(4)
    }

    fn cyan(&self) -> Styled<Self> {
        self.low_color(5)
    }

    fn light_gray(&self) -> Styled<Self> {
        self.low_color(6)
    }

    fn pink(&self) -> Styled<Self> {
        self.true_color(255, 100, 203)
    }

    fn light_green(&self) -> Styled<Self> {
        self.true_color(100, 255, 100)
    }

    fn light_red(&self) -> Styled<Self> {
        self.true_color(255, 80, 100)
    }

    fn gray(&self) -> Styled<Self> {
        self.true_color(128, 128, 128)
    }

    fn colorize_with(&self, with: &str) -> Styled<Self> {
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
        if with == "root" {
            self.red()
        } else {
            let hash = usize::from_str_radix(&sha256::digest(format!("{}\n", with))[..4], 16)
                .unwrap()
                % 115;
            let b = COLOR_TABLE[hash / 25];
            let g = COLOR_TABLE[hash / 5 % 5];
            let r = COLOR_TABLE[hash % 5];
            self.true_color(r, g, b)
        }
    }
}

impl<T: Display + ?Sized> Style for T {}
