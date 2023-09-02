use crate::{Environment, Icon, IconMode, Pretty, SimpleBlock, Style};
use nix::unistd;

pub struct RootShell(bool);

impl SimpleBlock for RootShell {
    fn extend(self: Box<Self>) -> Box<dyn Pretty> {
        self
    }
}

impl From<&Environment> for RootShell {
    fn from(_: &Environment) -> Self {
        RootShell(unistd::getuid().is_root())
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
        Some(
            if self.0 { icon.red() } else { icon.green() }
                .bold()
                .with_reset()
                .invisible()
                .to_string(),
        )
    }
}
