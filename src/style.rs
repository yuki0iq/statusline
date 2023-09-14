use std::fmt::{Display, Formatter, Result as FmtResult};

const INVISIBLE_START: &str = "\x01";
const INVISIBLE_END: &str = "\x02";
const ESC: &str = "\x1b";
const CSI: &str = "\x1b[";
const RESET: &str = "\x1b[0m";
const BEL: &str = "\x07";
const COLOR_TABLE: [u8; 5] = [255, 203, 153, 100, 0];
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
        match self.style {
            StyleKind::Title => write!(f, "{ESC}]0;{}{BEL}", self.value),
            StyleKind::Bold => write!(f, "{CSI}1m{}", self.value),
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
    /// There are 115 different colors. Here is the algorithm overview:
    ///
    /// ## Algorithm and reasoning behind it
    ///
    /// - Take first 4 hex numbers from sha256 digest of `with` with `'\n'` appended (later: take
    ///   first 5 digits, digest of string `"<with.len()> <with>"`)
    /// - Take modulo 115 as `color_number`
    /// - Let `color_table` be `[255, 203, 153, 100, 0]` with `FCA60` as a shorthand for its items
    /// - Let `green` be `color_number / 25`
    /// - Let `blue` be `color_number / 5 % 5`
    /// - Let `red` = `color_number % 5`
    ///
    /// `color_table` was chosen with a scientifick pick. It holds all of the possible values
    /// a single color channel can hold. The table was generated to make _all_ colors differ
    /// from each other.
    ///
    /// I have been experimenting with tables of size 6, but I was unlucky and did not find
    /// anything suitable and reverted to five colors.
    ///
    /// If the full table was used, then there would be 125 colors, but there are only 115.
    ///
    /// You may have noticed that `blue`, `green`, and `red` can be seen as a digits in base-5
    /// number. I will use the "FCA60" abbreviation for values in the color table --- these were
    /// chosen as a nearest hex number if value was divided by 16.
    ///
    /// I have examinated all these colors with a simple shell script (three `for` loops in some
    /// combinations to make sure everything's clear) and allowed only colors which were
    /// not too dark and not too red at the same time.
    ///
    /// All the colors were then marked as "allowed" or "disallowed" and similar patterns were
    /// merged into one. Here is the resulting table:
    ///
    /// |  Red  | Green |   Blue  | Allow?|
    /// |-------|-------|---------|-------|
    /// |  any  | not 0 |   any   |  yes  |
    /// |  any  |   0   | F, C, A |  yes  |
    /// |  any  |   0   | 6, 0    |  no   |
    ///
    /// The last line --- the only holding the disallowed colors --- was actually constructed some
    /// time later when I needed to convert the "number" to its color.
    ///
    /// If we order colors as in `F < C < A < 6 < 0` --- the order in which I have fiddled with
    /// the table --- we can see that only some of the "higher" numbers from blue and green channel
    /// are banned, and luckily one channel has only one banned item.
    ///
    /// 10 colors are banned. They, in GBR (strange one) look like `06_` and `00_`, where
    /// underscore denotes placeholder for every allowed value.
    ///
    /// Suppose we've written those numbers in big endian format. If we converted them to decimal
    /// values we would've got all the numbers from 115 to 124 --- the minimal is
    /// `06F = 4*25 + 3*5 + 0 = 115` and the maximal is `000 = 4*25 + 4*5 + 4 = 124`.
    ///
    /// Given this we can interpret our "color number" as a base-5 number and take green, blue,
    /// and red colors from it.
    fn colorize_with_prev(&self, with: &str) -> Styled<Self> {
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
            let hash =
                usize::from_str_radix(&sha256::digest(format!("{} {}", with.len(), with))[..5], 16)
                    .unwrap()
                    % 115;
            let g = COLOR_TABLE[hash / 25];
            let b = COLOR_TABLE[hash / 5 % 5];
            let r = COLOR_TABLE[hash % 5];
            self.true_color(r, g, b)
        }
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
    /// There are 24 different colors, simpler version with less colors
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
