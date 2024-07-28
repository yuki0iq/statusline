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
    Title,
    Bold,
    Italic,
    Color8(usize),
    TrueColor(u8, u8, u8),
    ResetEnd,
    Invisible,
    Visible,
    Boxed,
    Rounded,
    CursorHorizontalAbsolute(i32),
    CursorPreviousLine(i32),
    CursorSaveRestore,
    ClearLine,
    NewlineJoin(String),
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
/// ```
/// use statusline::Style;
///
/// let hello = "Hello world!";
/// let styled = hello.boxed().red().bold().with_reset().to_string();
/// //                ^^^^^^^^------=======.............
/// assert_eq!("\x1b[1m\x1b[31m[Hello world!]\x1b[0m", styled);
/// //          =======--------^            ^.......
/// ```
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
            StyleKind::Title => write!(f, "{ESC}]0;{}{BEL}", self.value),
            StyleKind::Bold => write!(f, "{CSI}1m{}", self.value),
            StyleKind::Italic => write!(f, "{CSI}3m{}", self.value),
            StyleKind::Color8(index) => write!(f, "{CSI}{}m{}", index + 31, self.value),
            StyleKind::TrueColor(r, g, b) => {
                write!(f, "{CSI}38;2;{r};{g};{b}m{}", self.value)
            }
            StyleKind::ResetEnd => write!(f, "{}{RESET}", self.value),
            StyleKind::Invisible => write!(f, "{INVISIBLE_START}{}{INVISIBLE_END}", self.value),
            StyleKind::Visible => write!(f, "{INVISIBLE_END}{}{INVISIBLE_START}", self.value),
            StyleKind::Boxed => write!(f, "[{}]", self.value),
            StyleKind::Rounded => write!(f, "({})", self.value),
            StyleKind::CursorHorizontalAbsolute(n) => write!(f, "{CSI}{n}G{}", self.value),
            StyleKind::CursorPreviousLine(n) => write!(f, "{CSI}{n}A{CSI}G{}", self.value),
            StyleKind::CursorSaveRestore => write!(f, "{CSI}s{}{CSI}u", self.value),
            StyleKind::ClearLine => write!(f, "{CSI}0K{}", self.value),
            StyleKind::NewlineJoin(s) => write!(f, "{}\n{s}", self.value),
        }
    }
}

/// Styling functions for function chaining
///
/// ```
/// use statusline::Style;
///
/// let hello = "Hello world!";
/// let styled = hello.boxed().red().bold().with_reset().to_string();
/// //                ^^^^^^^^------=======.............
/// assert_eq!("\x1b[1m\x1b[31m[Hello world!]\x1b[0m", styled);
/// //          =======--------^            ^.......
/// ```
///
/// Every chained function call a new [Styled] object wraps the previous result like a cabbage.
pub trait Style: Display {
    /// Format as a title for terminal
    ///
    /// ```
    /// use statusline::Style;
    /// assert_eq!("\x1b]0;yuki@reimu: /home/yuki\x07", "yuki@reimu: /home/yuki".as_title().to_string());
    /// ```
    fn as_title(&self) -> Styled<Self> {
        Styled {
            style: StyleKind::Title,
            value: self,
        }
    }

    /// Prepend bold style. Colors from 16-color palette may shift a bit
    ///
    /// ```
    /// use statusline::Style;
    /// assert_eq!("\x1b[1mBOLD text", "BOLD text".bold().to_string());
    /// ```
    fn bold(&self) -> Styled<Self> {
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
    fn italic(&self) -> Styled<Self> {
        Styled {
            style: StyleKind::Italic,
            value: self,
        }
    }

    /// Use colors from 16-color palette, dark version (0 for CSI 31 thru 6 for CSI 37, CSI 30 is black which
    /// is useless)
    fn low_color(&self, index: usize) -> Styled<Self> {
        Styled {
            style: StyleKind::Color8(index),
            value: self,
        }
    }

    /// Use true color. Note that some terminals lack true color support and will approximate
    /// the result with colors they do support. This may lead to text being completely unreadable.
    ///
    /// However, since most GUI terminal emulators in linux do support true color display no worry
    /// is usually needed. Just use it as-is
    fn true_color(&self, red: u8, green: u8, blue: u8) -> Styled<Self> {
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
    fn invisible(&self) -> Styled<Self> {
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
    fn visible(&self) -> Styled<Self> {
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
    fn with_reset(&self) -> Styled<Self> {
        Styled {
            style: StyleKind::ResetEnd,
            value: self,
        }
    }

    /// Wrap into square brackets
    ///
    /// ```
    /// use statusline::Style;
    /// assert_eq!("[nya]", "nya".boxed().to_string());
    /// ```
    fn boxed(&self) -> Styled<Self> {
        Styled {
            style: StyleKind::Boxed,
            value: self,
        }
    }

    /// Wrap into round brackets
    ///
    /// ```
    /// use statusline::Style;
    /// assert_eq!("(nyaah~)", "nyaah~".rounded().to_string());
    /// ```
    fn rounded(&self) -> Styled<Self> {
        Styled {
            style: StyleKind::Rounded,
            value: self,
        }
    }

    /// Set cursor position, the horizontal part, with absolute value. Coordinates are counted
    /// from 1, from line start to line end, which may seem counter-intuitive
    fn horizontal_absolute(&self, pos: i32) -> Styled<Self> {
        Styled {
            style: StyleKind::CursorHorizontalAbsolute(pos),
            value: self,
        }
    }

    /// Move cursor to the beginning of line which is `count` lines above the current one
    fn prev_line(&self, count: i32) -> Styled<Self> {
        Styled {
            style: StyleKind::CursorPreviousLine(count),
            value: self,
        }
    }

    /// Wrap into cursor saver --- for example for outputting PS1 above the PS1 "line"
    fn save_restore(&self) -> Styled<Self> {
        Styled {
            style: StyleKind::CursorSaveRestore,
            value: self,
        }
    }

    /// Prepends line cleaner
    fn clear_till_end(&self) -> Styled<Self> {
        Styled {
            style: StyleKind::ClearLine,
            value: self,
        }
    }

    /// Join current line with fixed one with newline
    fn join_lf(&self, s: String) -> Styled<Self> {
        Styled {
            style: StyleKind::NewlineJoin(s),
            value: self,
        }
    }

    /// Red color from 16-color palette (CSI 31)
    fn red(&self) -> Styled<Self> {
        self.low_color(0)
    }

    /// Green color from 16-color palette (CSI 32)
    fn green(&self) -> Styled<Self> {
        self.low_color(1)
    }

    /// Yellow color from 16-color palette (CSI 33)
    fn yellow(&self) -> Styled<Self> {
        self.low_color(2)
    }

    /// Blue color from 16-color palette (CSI 34)
    fn blue(&self) -> Styled<Self> {
        self.low_color(3)
    }

    /// Purple color from 16-color palette (CSI 35)
    fn purple(&self) -> Styled<Self> {
        self.low_color(4)
    }

    /// Cyan color from 16-color palette (CSI 36)
    fn cyan(&self) -> Styled<Self> {
        self.low_color(5)
    }

    /// Light gray color from 16-color palette (CSI 37)
    fn light_gray(&self) -> Styled<Self> {
        self.low_color(6)
    }

    /// Pink color (true)
    fn pink(&self) -> Styled<Self> {
        self.true_color(255, 100, 203)
    }

    /// Light green color (true)
    fn light_green(&self) -> Styled<Self> {
        self.true_color(100, 255, 100)
    }

    /// Light red color (true)
    fn light_red(&self) -> Styled<Self> {
        self.true_color(255, 80, 100)
    }

    /// Gray color (true)
    fn gray(&self) -> Styled<Self> {
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
    fn colorize_with(&self, with: &str) -> Styled<Self> {
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
