use crate::{Environment, Icon, Icons, Pretty, Style, Styled};

pub enum ReturnCode {
    Ok,
    Failed,
    NotAvailable,
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

impl ReturnCode {
    fn icon<'a>(&'a self, icons: &Icons) -> Styled<'a, str> {
        match &self {
            Self::Ok => icons(Icon::ReturnOk),
            Self::Failed => icons(Icon::ReturnFail),
            Self::NotAvailable => icons(Icon::ReturnNA),
        }
        .visible()
    }
}

impl Pretty for ReturnCode {
    fn pretty(&self, icons: &Icons) -> Option<String> {
        let icon = self.icon(icons);
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
