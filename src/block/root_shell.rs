use crate::{Environment, Extend, Icon, IconMode, Pretty, Style as _};
use rustix::process;
use std::borrow::Cow;

pub struct RootShell(bool, usize);

impl Extend for RootShell {
    fn extend(self: Box<Self>) -> Box<dyn Pretty> {
        self
    }
}

impl From<&Environment> for RootShell {
    fn from(_: &Environment) -> Self {
        RootShell(
            process::getuid().is_root(),
            std::env::var("SHLVL")
                .unwrap_or_default()
                .parse()
                .unwrap_or_default(),
        )
    }
}

impl Icon for RootShell {
    fn icon(&self, _: IconMode) -> &'static str {
        if self.0 { "#" } else { "$" }
    }
}

impl Pretty for RootShell {
    fn pretty(&self, mode: IconMode) -> Option<String> {
        let icon = self.icon(mode);
        let shlvl = if self.1 > 0 {
            Cow::from((1 + self.1).to_string())
        } else {
            Cow::from("")
        };
        let formatted = format!("{shlvl}{icon}");
        let formatted = formatted.visible();
        Some(
            if self.0 {
                formatted.red()
            } else {
                formatted.green()
            }
            .with_reset()
            .invisible()
            .to_string(),
        )
    }
}
