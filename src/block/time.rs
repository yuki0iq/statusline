use crate::{Block, IconMode, Pretty, Style as _};
use chrono::prelude::*;

pub struct Time(DateTime<Local>);

impl Block for Time {}

impl Time {
    pub fn new() -> Box<dyn Block> {
        Box::new(Self(Local::now()))
    }
}

impl Pretty for Time {
    fn pretty(&self, _: IconMode) -> Option<String> {
        Some(
            self.0
                .format("%a, %Y-%b-%d, %H:%M:%S in %Z")
                .visible()
                .gray()
                .with_reset()
                .invisible()
                .to_string(),
        )
    }
}
