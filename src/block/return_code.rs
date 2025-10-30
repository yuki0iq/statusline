use crate::{Environment, Extend, Icon, IconMode, Pretty, Style as _};
use linux_raw_sys::general::{_NSIG as SIGRTMAX, SIGRTMIN};
use rustix::process::Signal;

pub enum ReturnCode {
    Ok,
    Failed(u8),
    Signaled(String),
    NotAvailable,
}

impl Extend for ReturnCode {
    fn extend(self: Box<Self>) -> Box<dyn Pretty> {
        self
    }
}

impl ReturnCode {
    pub fn new(args: &Environment) -> Box<Self> {
        Box::new(match args.ret_code {
            Some(0) => Self::Ok,
            None => Self::NotAvailable,
            Some(code) => match signal_name(code.wrapping_sub(128)) {
                Some(sig) => Self::Signaled(sig),
                None => Self::Failed(code),
            },
        })
    }
}

impl Icon for ReturnCode {
    fn icon(&self, mode: IconMode) -> &'static str {
        use IconMode::*;
        match &self {
            Self::Ok => match &mode {
                Icons => "✓",
                Text | MinimalIcons => "",
            },
            Self::Failed(..) => "",
            Self::Signaled(..) => match &mode {
                Icons => "󰜃 ",
                Text | MinimalIcons => "",
            },
            Self::NotAvailable => match &mode {
                Text => "N/A",
                Icons | MinimalIcons => "⁇",
            },
        }
    }
}

impl Pretty for ReturnCode {
    fn pretty(&self, mode: IconMode) -> Option<String> {
        let icon = self.icon(mode);
        let text = match &self {
            Self::Ok | Self::NotAvailable => icon.into(),
            // 126 not exec
            // 127 not found
            Self::Failed(code) => format!("{code}{icon}"),
            Self::Signaled(sig) => format!("{icon}{sig}"),
        };
        if text.is_empty() {
            None?;
        }
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
    let sig = u32::from(sig);
    if (SIGRTMIN..=SIGRTMAX).contains(&sig) {
        Some(format!("RT{}", sig - SIGRTMIN))
    } else if sig == 0 || sig > SIGRTMAX {
        None
    } else {
        // SAFETY: No realtime signals are here; result is used only for pretty-printing
        let sig = unsafe { Signal::from_raw_unchecked(sig.cast_signed()) };
        let pretty = format!("{sig:?}"); // "Signal::TERM"
        Some(pretty[9..pretty.len() - 1].to_ascii_uppercase())
    }
}
