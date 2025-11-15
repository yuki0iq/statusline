use crate::{Block, Color, Environment, Icon, IconMode, Pretty, Style, WithStyle as _};

pub struct RootShell {
    is_root: bool,
    depth: usize,
}

super::register_block!(RootShell);

impl Block for RootShell {
    fn new(_: &Environment) -> Option<Box<dyn Block>> {
        Some(Box::new(RootShell {
            is_root: rustix::process::getuid().is_root(),
            depth: std::env::var("SHLVL")
                .unwrap_or_default()
                .parse()
                .unwrap_or_default(),
        }))
    }
}

impl Icon for RootShell {
    fn icon(&self, _: IconMode) -> &'static str {
        if self.is_root { "#" } else { "$" }
    }
}

impl Pretty for RootShell {
    fn pretty(&self, f: &mut std::fmt::Formatter<'_>, mode: IconMode) -> std::fmt::Result {
        let color = if self.is_root {
            Color::RED
        } else {
            Color::GREEN
        };

        f.with_style(color, Style::empty(), |f| {
            if self.depth > 0 {
                write!(f, "{}", 1 + self.depth)?;
            }
            write!(f, "{}", self.icon(mode))
        })
    }
}
