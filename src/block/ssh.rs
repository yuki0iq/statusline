use crate::{
    Block, Color, Environment, Icon, IconMode, Pretty, Style, WithStyle as _,
    workgroup::{SshChain, WorkgroupKey},
};

pub struct Ssh(Vec<String>);

super::register_block!(Ssh);

impl Block for Ssh {
    fn new(_: &Environment) -> Option<Self> {
        let chain = SshChain::open(WorkgroupKey::load().ok().as_ref()).0;
        (!chain.is_empty()).then_some(Ssh(chain))
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
        f.with_style(Color::CYAN, Style::empty(), |f| {
            write!(f, "[{}", self.icon(mode))?;
            for link in &self.0 {
                write!(f, " {link}")?;
            }
            write!(f, "]")
        })
    }
}
