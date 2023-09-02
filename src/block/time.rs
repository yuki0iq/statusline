use crate::{Environment, IconMode, Pretty, SimpleBlock, Style};
use chrono::prelude::*;

pub type Time = DateTime<Local>;

impl SimpleBlock for Time {
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
        let term_width = term_size::dimensions().map(|s| s.0).unwrap_or(80) as i32;
        let datetime = datetime_str
            .gray()
            .with_reset()
            .horizontal_absolute(term_width - datetime_str.len() as i32)
            .to_string();
        Some(datetime)
    }
}
