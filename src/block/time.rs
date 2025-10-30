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
    fn pretty(&self, _: IconMode) -> Option<String> {
        Some(
            self.format("%a, %Y-%b-%d, %H:%M:%S in %Z")
                .visible()
                .gray()
                .with_reset()
                .invisible()
                .to_string(),
        )
    }
}
