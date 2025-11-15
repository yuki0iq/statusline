use crate::{Block, Color, Environment, IconMode, Pretty, Style, WithStyle as _};
use chrono::prelude::*;

pub struct Time(DateTime<Local>);

super::register_block!(Time);

impl Block for Time {
    fn new(_: &Environment) -> Option<Box<dyn Block>> {
        Some(Box::new(Self(Local::now())))
    }
}

impl Pretty for Time {
    fn pretty(&self, f: &mut std::fmt::Formatter<'_>, _: IconMode) -> std::fmt::Result {
        f.with_style(Color::GRAY, Style::empty(), |f| {
            write!(f, "{}", self.0.format("%a, %Y-%b-%d, %H:%M:%S in %Z"))
        })
    }
}
