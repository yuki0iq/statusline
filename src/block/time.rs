use crate::{Block, Environment, IconMode, Pretty, Style as _};
use chrono::prelude::*;

pub struct Time(DateTime<Local>);

impl Block for Time {
    fn new(_: &Environment) -> Option<Box<dyn Block>> {
        Some(Box::new(Self(Local::now())))
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
