use crate::{Environment, Icon, IconMode, Pretty, SimpleBlock, Style};
use std::env;

pub struct Ssh(Option<String>);

impl From<&Environment> for Ssh {
    fn from(_: &Environment) -> Ssh {
        Ssh(env::var("SSH_CONNECTION")
            .ok()
            .and_then(|x| x.split_whitespace().next().map(|x| x.to_string())))
    }
}

impl SimpleBlock for Ssh {
    fn extend(self: Box<Self>) -> Box<dyn Pretty> {
        self
    }
}

impl Icon for Ssh {
    fn icon(&self, mode: &IconMode) -> &'static str {
        use IconMode::*;
        match mode {
            Text => "ssh",
            Icons | MinimalIcons => "󰌘",
        }
    }
}

impl Pretty for Ssh {
    fn pretty(&self, mode: &IconMode) -> Option<String> {
        let ip = self.0.as_ref()?;
        let icon = self.icon(mode);
        Some(
            format!("{icon} {ip}")
                .boxed()
                .visible()
                .cyan()
                .invisible()
                .to_string(),
        )
    }
}
