use crate::{Environment, Icon, IconMode, Pretty, SimpleBlock, Style};
use nix::unistd;
use std::{borrow::Cow, env};

pub struct RootShell(bool, usize);

impl SimpleBlock for RootShell {
    fn extend(self: Box<Self>) -> Box<dyn Pretty> {
        self
    }
}

impl From<&Environment> for RootShell {
    fn from(_: &Environment) -> Self {
        RootShell(
            unistd::getuid().is_root(),
            env::var("SHLVL")
                .unwrap_or_default()
                .parse()
                .unwrap_or_default(),
        )
    }
}

impl Icon for RootShell {
    fn icon(&self, _: &IconMode) -> &'static str {
        if self.0 {
            "#"
        } else {
            "$"
        }
    }
}

impl Pretty for RootShell {
    fn pretty(&self, mode: &IconMode) -> Option<String> {
        let icon = self.icon(mode).visible();
        let shlvl = if self.1 > 1 {
            Cow::from(self.1.to_string())
        } else {
            Cow::from("")
        };
        let formatted = format!("{shlvl}{icon}");
        Some(
            if self.0 {
                formatted.red()
            } else {
                formatted.green()
            }
            .bold()
            .with_reset()
            .invisible()
            .to_string(),
        )
    }
}
