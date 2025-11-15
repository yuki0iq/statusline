use std::time::Duration;

use crate::{Block, Color, Environment, Icon, IconMode, Pretty, Style, WithStyle as _};

pub struct Elapsed(Duration);

super::register_block!(Elapsed);

impl Block for Elapsed {
    fn new(environ: &Environment) -> Option<Self> {
        let elapsed = environ.elapsed_time.unwrap_or_default();
        (elapsed > Duration::from_millis(100)).then_some(Elapsed(elapsed))
    }
}

impl Icon for Elapsed {
    fn icon(&self, mode: IconMode) -> &'static str {
        use IconMode::*;
        match &mode {
            Text => "took",
            Icons | MinimalIcons => "",
        }
    }
}

impl Pretty for Elapsed {
    fn pretty(&self, f: &mut std::fmt::Formatter<'_>, mode: IconMode) -> std::fmt::Result {
        f.with_style(Color::CYAN, Style::empty(), |f| {
            write!(
                f,
                "({} {})",
                self.icon(mode),
                milliseconds_to_string(self.0.as_millis())
            )
        })
    }
}

fn milliseconds_to_string(total: u128) -> String {
    let (msec, total) = (total % 1000, total / 1000);
    let (sec, total) = (total % 60, total / 60);
    let (min, total) = (total % 60, total / 60);
    let (hrs, total) = (total % 24, total / 24);
    let (day, week) = (total % 7, total / 7);

    let times = [
        (week, "w"),
        (day, "d"),
        (hrs, "h"),
        (min, "m"),
        (sec, "s"),
        (msec, "ms"),
    ];
    let mut iter = times.iter().peekable();
    while let Some((0, _)) = iter.peek() {
        iter.next();
    }

    iter.take(2)
        .map(|(val, ch)| val.to_string() + ch)
        .collect::<Vec<_>>()
        .join(" ")
}
