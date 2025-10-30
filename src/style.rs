use const_format::formatcp;
use std::fmt::{Display, Formatter, Result as FmtResult};

const INVISIBLE_START: &str = "\x01";
const INVISIBLE_END: &str = "\x02";
const ESC: &str = "\x1b";
const CSI: &str = "\x1b[";
const RESET: &str = "\x1b[0m";
const BEL: &str = "\x07";
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

enum StyleKind {
    Bold,
    Italic,
    Color8(usize),
    Color16(usize),
    TrueColor(u8, u8, u8),
    ResetEnd,
    Invisible,
    Visible,
}

/// Styled "string"-like object
///
/// Can be used instead of raw ANSI sequences because of cleaner interface.
///
/// This type is a wrapper around any object which implements [Display] trait --- usually some
/// kind of strings or other simple ogjects. It wraps one style change at a time, making
/// it possible to chain styles to apply them from the innermost to the outermost.
///
/// All the magic happens in [Style] trait.
///
/// This wrapper applies styles only when formatted to string. Formatting results are never saved
/// and every call to `<Styled as ToString>::to_string()` will format the result one more time,
/// which may lead to different results for special types like the one which tells the exact
/// moment of time at the `std::fmt::Display::fmt` call.
pub struct Styled<'a, T: Display + ?Sized> {
    style: StyleKind,
    value: &'a T,
}

impl<T: Display + ?Sized> Display for Styled<'_, T> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match &self.style {
            StyleKind::Bold => write!(f, "{CSI}1m{}", self.value),
            StyleKind::Italic => write!(f, "{CSI}3m{}", self.value),
            StyleKind::Color8(index) => write!(f, "{CSI}{}m{}", index + 31, self.value),
            StyleKind::Color16(index) => write!(f, "{CSI}{}m{}", index + 90, self.value),
            StyleKind::TrueColor(r, g, b) => {
                write!(f, "{CSI}38;2;{r};{g};{b}m{}", self.value)
            }
            StyleKind::ResetEnd => write!(f, "{}{RESET}", self.value),
            StyleKind::Invisible => write!(f, "{INVISIBLE_START}{}{INVISIBLE_END}", self.value),
            StyleKind::Visible => write!(f, "{INVISIBLE_END}{}{INVISIBLE_START}", self.value),
        }
    }
}

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
    format!("{INVISIBLE_START}{ESC}]0;{title}{BEL}{INVISIBLE_END}")
}
pub fn horizontal_absolute(n: usize) -> String {
    format!("{INVISIBLE_START}{CSI}{n}G{INVISIBLE_END}")
}

/// Styling functions for function chaining
///
/// ```
/// use statusline::Style;
///
/// let hello = "Hello world!";
/// let styled = hello.red().bold().with_reset().to_string();
/// //                ------=======.............
/// assert_eq!("\x1b[1m\x1b[31mHello world!\x1b[0m", styled);
/// //          =======--------            .......
/// ```
///
/// Every chained function call a new [Styled] object wraps the previous result like a cabbage.
pub trait Style: Display {
    /// Prepend bold style. Colors from 16-color palette may shift a bit
    ///
    /// ```
    /// use statusline::Style;
    /// assert_eq!("\x1b[1mBOLD text", "BOLD text".bold().to_string());
    /// ```
    fn bold(&self) -> Styled<'_, Self> {
        Styled {
            style: StyleKind::Bold,
            value: self,
        }
    }

    /// Prepend italic style
    ///
    /// ```
    /// use statusline::Style;
    /// assert_eq!("\x1b[3mItalic text", "Italic text".italic().to_string());
    /// ```
    fn italic(&self) -> Styled<'_, Self> {
        Styled {
            style: StyleKind::Italic,
            value: self,
        }
    }

    /// Use colors from 16-color palette, dark version (0 for CSI 31 thru 6 for CSI 37, CSI 30 is black which
    /// is useless)
    fn low_color(&self, index: usize) -> Styled<'_, Self> {
        Styled {
            style: StyleKind::Color8(index),
            value: self,
        }
    }

    /// Use colors from 16-color palette, light version (0 for CSI 90 thru 7 for CSI 97, CSI 90 is black which
    /// is useless)
    fn high_color(&self, index: usize) -> Styled<'_, Self> {
        Styled {
            style: StyleKind::Color16(index),
            value: self,
        }
    }

    /// Use true color. Note that some terminals lack true color support and will approximate
    /// the result with colors they do support. This may lead to text being completely unreadable.
    ///
    /// However, since most GUI terminal emulators in linux do support true color display no worry
    /// is usually needed. Just use it as-is
    fn true_color(&self, red: u8, green: u8, blue: u8) -> Styled<'_, Self> {
        Styled {
            style: StyleKind::TrueColor(red, green, blue),
            value: self,
        }
    }

    /// Wrap into "readline invisible" characters, for PS1 output or some other strange things
    ///
    /// ```
    /// use statusline::Style;
    /// assert_eq!("\x01invis\x02", "invis".invisible().to_string());
    /// ```
    fn invisible(&self) -> Styled<'_, Self> {
        Styled {
            style: StyleKind::Invisible,
            value: self,
        }
    }

    /// Wrap into "readline invisible" but reverse --- for making surroundings invisible.
    ///
    /// ```
    /// use statusline::Style;
    /// assert_eq!("\x01\x1b[31m\x02Visible\x01\x1b[0m\x02",
    ///     "Visible".visible().red().with_reset().invisible().to_string());
    /// ```
    fn visible(&self) -> Styled<'_, Self> {
        Styled {
            style: StyleKind::Visible,
            value: self,
        }
    }

    /// Add "reset colors and boldness" to the end
    ///
    /// ```
    /// use statusline::Style;
    /// assert_eq!("\x1b[31mRED\x1b[0mnormal", "RED".red().with_reset().to_string() + "normal");
    /// ```
    fn with_reset(&self) -> Styled<'_, Self> {
        Styled {
            style: StyleKind::ResetEnd,
            value: self,
        }
    }

    /// Red color from 16-color palette (CSI 31)
    fn red(&self) -> Styled<'_, Self> {
        self.low_color(0)
    }

    /// Green color from 16-color palette (CSI 32)
    fn green(&self) -> Styled<'_, Self> {
        self.low_color(1)
    }

    /// Yellow color from 16-color palette (CSI 33)
    fn yellow(&self) -> Styled<'_, Self> {
        self.low_color(2)
    }

    /// Blue color from 16-color palette (CSI 34)
    fn blue(&self) -> Styled<'_, Self> {
        self.low_color(3)
    }

    /// Purple color from 16-color palette (CSI 35)
    fn purple(&self) -> Styled<'_, Self> {
        self.low_color(4)
    }

    /// Cyan color from 16-color palette (CSI 36)
    fn cyan(&self) -> Styled<'_, Self> {
        self.low_color(5)
    }

    /// Light gray color from 16-color palette (CSI 37)
    fn light_gray(&self) -> Styled<'_, Self> {
        self.low_color(6)
    }

    /// Bright blue color from 16-color palette (CSI 94)
    fn bright_blue(&self) -> Styled<'_, Self> {
        self.high_color(4)
    }

    /// Pink color (true)
    fn pink(&self) -> Styled<'_, Self> {
        self.true_color(255, 100, 203)
    }

    /// Light green color (true)
    fn light_green(&self) -> Styled<'_, Self> {
        self.true_color(100, 255, 100)
    }

    /// Light red color (true)
    fn light_red(&self) -> Styled<'_, Self> {
        self.true_color(255, 80, 100)
    }

    /// Gray color (true)
    fn gray(&self) -> Styled<'_, Self> {
        self.true_color(128, 128, 128)
    }

    /// String autocolorizer.
    ///
    /// Colors `self` with a "random" color associated with given string `with`.
    ///
    /// |`with` value|Resulting color     |
    /// |------------|--------------------|
    /// |`="root"`   | Red                |
    /// |other       | Some non-red color |
    ///
    /// There are 24 different colors
    fn colorize_with(&self, with: &str) -> Styled<'_, Self> {
        if with == "root" {
            self.red()
        } else {
            let idx = polyhash(with, 23, 179, with.len()) + 1;
            let (r, g, b) = HSV_COLOR_TABLE[idx];
            self.true_color(r, g, b)
        }
    }
}

/// All types which can be displayed can be styled too
impl<T: Display + ?Sized> Style for T {}

fn polyhash(s: &str, m: usize, p: usize, h_init: usize) -> usize {
    let mut h = h_init % m;
    for by in s.bytes() {
        h = (h * p + by as usize) % m;
    }
    h
}
