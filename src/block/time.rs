use crate::{Environment, Extend, IconMode, Pretty, Style as _};
use chrono::prelude::*;

pub type Time = DateTime<Local>;

impl Extend for Time {
    fn extend(self: Box<Self>) -> Box<dyn Pretty> {
        self
    }
}

impl From<&Environment> for Time {
    fn from(_: &Environment) -> Self {
        Local::now()
    }
}

impl Pretty for Time {
    fn pretty(&self, _: &IconMode) -> Option<String> {
        let datetime_str = self.format("%a, %Y-%b-%d, %H:%M:%S in %Z").to_string();
        let term_width = terminal_size::terminal_size().map_or(80, |(w, _h)| w.0) as usize;
        let hpos = term_width.saturating_sub(datetime_str.len());
        let datetime = datetime_str
            .visible()
            .gray()
            .with_reset()
            .horizontal_absolute(hpos)
            .invisible()
            .to_string();
        Some(datetime)
    }
}
