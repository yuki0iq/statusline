use crate::{Block, Environment, Icon, IconMode, Pretty, Style as _};
use std::borrow::Cow;

pub struct RootShell(bool, usize);

super::register_block!(RootShell);

impl Block for RootShell {
    fn new(_: &Environment) -> Option<Box<dyn Block>> {
        Some(Box::new(RootShell(
            rustix::process::getuid().is_root(),
            std::env::var("SHLVL")
                .unwrap_or_default()
                .parse()
                .unwrap_or_default(),
        )))
    }
}

impl Icon for RootShell {
    fn icon(&self, _: IconMode) -> &'static str {
        if self.0 { "#" } else { "$" }
    }
}

impl Pretty for RootShell {
    fn pretty(&self, f: &mut std::fmt::Formatter<'_>, mode: IconMode) -> std::fmt::Result {
        let icon = self.icon(mode);
        let shlvl = if self.1 > 0 {
            Cow::from((1 + self.1).to_string())
        } else {
            Cow::from("")
        };
        let formatted = format!("{shlvl}{icon}");
        let formatted = formatted.visible();
        write!(
            f,
            "{}",
            if self.0 {
                formatted.red()
            } else {
                formatted.green()
            }
            .with_reset()
            .invisible(),
        )
    }
}
