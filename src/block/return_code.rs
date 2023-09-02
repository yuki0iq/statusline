use crate::{Environment, Icon, IconMode, Pretty, SimpleBlock, Style};

pub enum ReturnCode {
    Ok,
    Failed,
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
            Some(0) | Some(130) => Self::Ok,
            Some(_) => Self::Failed,
            None => Self::NotAvailable,
        }
    }
}

impl Icon for ReturnCode {
    fn icon(&self, mode: &IconMode) -> &'static str {
        use IconMode::*;
        match &self {
            Self::Ok => match &mode {
                Text => "OK",
                Icons | MinimalIcons => "✓",
            },
            Self::Failed => match &mode {
                Text => "Failed",
                Icons | MinimalIcons => "✗",
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
        let icon = self.icon(mode).visible();
        Some(
            match &self {
                Self::Ok => icon.light_green(),
                Self::Failed => icon.light_red(),
                Self::NotAvailable => icon.light_gray(),
            }
            .with_reset()
            .invisible()
            .to_string(),
        )
    }
}
