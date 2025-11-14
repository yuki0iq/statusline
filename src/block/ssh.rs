use crate::{
    Block, Environment, Icon, IconMode, Pretty, Style as _,
    workgroup::{SshChain, WorkgroupKey},
};

pub struct Ssh(String);

super::register_block!(Ssh);

impl Block for Ssh {
    fn new(_: &Environment) -> Option<Box<dyn Block>> {
        let chain = SshChain::open(WorkgroupKey::load().ok().as_ref()).0;
        if chain.is_empty() {
            None
        } else {
            Some(Box::new(Ssh(chain.join(" "))))
        }
    }
}

impl Icon for Ssh {
    fn icon(&self, mode: IconMode) -> &'static str {
        use IconMode::*;
        match mode {
            Text => "ssh",
            Icons | MinimalIcons => "󰌘",
        }
    }
}

impl Pretty for Ssh {
    fn pretty(&self, f: &mut std::fmt::Formatter<'_>, mode: IconMode) -> std::fmt::Result {
        let chain = &self.0;
        let icon = self.icon(mode);
        write!(
            f,
            "{}",
            format!("[{icon} {chain}]").visible().cyan().invisible()
        )
    }
}
