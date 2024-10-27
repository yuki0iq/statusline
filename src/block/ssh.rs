use crate::{
    workgroup::{SshChain, WorkgroupKey},
    Environment, Extend, Icon, IconMode, Pretty, Style as _,
};

pub struct Ssh(String);

impl From<&Environment> for Ssh {
    fn from(_: &Environment) -> Ssh {
        Ssh(SshChain::open(WorkgroupKey::load().ok().as_ref())
            .0
            .join(" "))
    }
}

impl Extend for Ssh {
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
        let chain = &self.0;
        if chain.is_empty() {
            return None;
        }

        let icon = self.icon(mode);
        Some(
            format!("{icon} {chain}")
                .boxed()
                .visible()
                .cyan()
                .invisible()
                .to_string(),
        )
    }
}
