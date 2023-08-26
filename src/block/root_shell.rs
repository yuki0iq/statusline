use crate::{Environment, FromEnv, Icons, Pretty, Style, Styled};
use nix::unistd;

pub struct RootShell(bool);

impl FromEnv for RootShell {
    fn from_env(_: &Environment) -> Self {
        RootShell(unistd::getuid().is_root())
    }
}

impl RootShell {
    fn icon(&self) -> Styled<'_, str> {
        if self.0 { "#" } else { "$" }.visible()
    }
}

impl Pretty for RootShell {
    fn pretty(&self, _: &Icons) -> Option<String> {
        let icon = self.icon();
        Some(
            if self.0 { icon.red() } else { icon.green() }
                .bold()
                .with_reset()
                .invisible()
                .to_string(),
        )
    }
}
