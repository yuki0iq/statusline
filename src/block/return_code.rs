use crate::{Environment, Icon, IconMode, Pretty, SimpleBlock, Style};
use rustix::process::Signal;

pub enum ReturnCode {
    Ok,
    Failed(u8),
    Signaled(String),
    NotAvailable,
}

impl SimpleBlock for ReturnCode {
    fn extend(self: Box<Self>) -> Box<dyn Pretty> {
        self
    }
}

impl From<&Environment> for ReturnCode {
    fn from(args: &Environment) -> Self {
        match args.ret_code {
            Some(0) => Self::Ok,
            None => Self::NotAvailable,
            Some(code) => match signal_name(code.wrapping_sub(128)) {
                Some(sig) => Self::Signaled(sig),
                None => Self::Failed(code),
            },
        }
    }
}

impl Icon for ReturnCode {
    fn icon(&self, mode: &IconMode) -> &'static str {
        use IconMode::*;
        match &self {
            Self::Ok => match &mode {
                Text => "OK",
                Icons => "✓",
                MinimalIcons => "",
            },
            Self::Failed(..) => "",
            Self::Signaled(..) => match &mode {
                Text => "SIG",
                Icons => "󰜃 ",
                MinimalIcons => "",
            },
            Self::NotAvailable => match &mode {
                Text => "N/A",
                Icons | MinimalIcons => "⁇",
            },
        }
    }
}

impl Pretty for ReturnCode {
    fn pretty(&self, mode: &IconMode) -> Option<String> {
        let icon = self.icon(mode).to_string();
        let text = match &self {
            Self::Ok | Self::NotAvailable => icon,
            // 126 not exec
            // 127 not found
            Self::Failed(code) => format!("{code}{icon}"),
            Self::Signaled(sig) => format!("{icon}{sig}"),
        };
        if text.is_empty() {
            None?
        };
        let text = text.visible();

        Some(
            match &self {
                Self::Ok => text.light_green(),
                Self::Failed(..) => text.light_red(),
                Self::Signaled(..) => text.true_color(255, 170, 0),
                Self::NotAvailable => text.light_gray(),
            }
            .with_reset()
            .invisible()
            .to_string(),
        )
    }
}

fn signal_name(sig: u8) -> Option<String> {
    let sig = sig as i32;
    if let Some(sig) = Signal::from_raw(sig) {
        Some(format!("{:?}", sig).to_ascii_uppercase())
    } else if (libc::SIGRTMIN()..=libc::SIGRTMAX()).contains(&sig) {
        Some(format!("RT{}", sig - libc::SIGRTMIN()))
    } else {
        None
    }
}
