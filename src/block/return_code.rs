use crate::{Block, Color, Environment, Icon, IconMode, Pretty, Style, WithStyle as _};
use linux_raw_sys::general::{_NSIG as SIGRTMAX, SIGRTMIN};
use rustix::process::Signal;

pub enum ReturnCode {
    Ok,
    Failed(u8),
    Signaled(String),
    NotAvailable,
}

super::register_block!(ReturnCode);

impl Block for ReturnCode {
    fn new(environ: &Environment) -> Option<Box<dyn Block>> {
        // Additional codes worth considering: 126 not exec, 127 not found
        Some(Box::new(match environ.ret_code {
            Some(0) => Self::Ok,
            None => Self::NotAvailable,
            Some(code) => match signal_name(code.wrapping_sub(128)) {
                Some(sig) => Self::Signaled(sig),
                None => Self::Failed(code),
            },
        }))
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
    fn pretty(&self, f: &mut std::fmt::Formatter<'_>, mode: IconMode) -> std::fmt::Result {
        let icon = self.icon(mode);

        let color = match &self {
            Self::Ok => Color::LIGHT_GREEN,
            Self::Failed(..) => Color::LIGHT_RED,
            Self::Signaled(..) => Color::TRUE_YELLOW,
            Self::NotAvailable => Color::LIGHT_GRAY,
        };

        if let Self::Ok | Self::NotAvailable = self
            && icon.is_empty()
        {
            return Ok(());
        }

        f.with_style(color, Style::empty(), |f| match &self {
            Self::Ok | Self::NotAvailable => write!(f, "{icon}"),
            Self::Failed(code) => write!(f, "{code}{icon}"),
            Self::Signaled(sig) => write!(f, "{icon}{sig}"),
        })
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
