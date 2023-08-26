use crate::Chassis;
use std::env;

/// Icons' modes
pub enum Icons {
    /// Use text instead of icons
    Text,
    /// Use icons from nerdfonts
    Icons,
    /// Use alternative icon set (simpler icons, but sometimes hard to get the meaning)
    MinimalIcons,
}

impl Icons {
    /// Detect prompt mode from `PS1_MODE` environment variable
    ///
    /// | Environment        | Resulting IconMode   |
    /// |--------------------|----------------------|
    /// | `PS1_MODE=text`    | Text                 |
    /// | `PS1_MODE=minimal` | Alternative nerdfont |
    /// | otherwise          | Default nerdfont     |
    pub fn build() -> Self {
        match env::var("PS1_MODE") {
            Ok(x) if x == "text" => Icons::Text,
            Ok(x) if x == "minimal" => Icons::MinimalIcons,
            _ => Icons::Icons,
        }
    }
}

impl FnOnce<(Icon,)> for Icons {
    type Output = &'static str;
    extern "rust-call" fn call_once(self, args: (Icon,)) -> Self::Output {
        args.0.static_pretty(&self)
    }
}

impl FnMut<(Icon,)> for Icons {
    extern "rust-call" fn call_mut(&mut self, args: (Icon,)) -> Self::Output {
        args.0.static_pretty(self)
    }
}

impl Fn<(Icon,)> for Icons {
    extern "rust-call" fn call(&self, args: (Icon,)) -> Self::Output {
        args.0.static_pretty(self)
    }
}

pub trait Pretty {
    fn pretty(&self, icons: &Icons) -> String;
}

/// TODO
#[non_exhaustive]
pub enum Icon {
    Host,
    User,
    HostAt,
    ReadOnly,
    OnBranch,
    AtCommit,
    Ahead,
    Behind,
    Stashes,
    Conflict,
    Staged,
    Dirty,
    Untracked,
    Bisecting,
    Reverting,
    CherryPicking,
    Merging,
    Rebasing,
    ReturnOk,
    ReturnFail,
    ReturnNA,
    TookTime,
    Venv,
}

impl Icon {
    fn static_pretty(&self, icons: &Icons) -> &'static str {
        use self::{
            Icon::*,
            Icons::{Icons, MinimalIcons, Text},
        };
        match &self {
            Host => match &icons {
                Text => "",
                Icons | MinimalIcons => Chassis::get().icon(),
            },
            User => match &icons {
                Text => "as",
                Icons | MinimalIcons => "",
            },
            HostAt => match &icons {
                Text => " at ",
                Icons | MinimalIcons => "＠",
            },
            ReadOnly => match &icons {
                Text => "R/O",
                Icons | MinimalIcons => "",
            },
            OnBranch => match &icons {
                Text => "on",
                Icons | MinimalIcons => "",
            },
            AtCommit => match &icons {
                Text => "at",
                Icons | MinimalIcons => "",
            },
            Ahead => match &icons {
                Text => "^",
                Icons | MinimalIcons => "󰞙 ",
            },
            Behind => match &icons {
                Text => "v",
                Icons | MinimalIcons => "󰞕 ",
            },
            Stashes => match &icons {
                Text => "*",
                Icons | MinimalIcons => " ",
            },
            Conflict => match &icons {
                Text => "~",
                Icons => "󰞇 ",
                MinimalIcons => " ",
            },
            Staged => match &icons {
                Text => "+",
                Icons | MinimalIcons => " ",
            },
            Dirty => match &icons {
                Text => "!",
                Icons | MinimalIcons => " ",
            },
            Untracked => match &icons {
                Text => "?",
                Icons => " ",
                MinimalIcons => " ",
            },
            Bisecting => match &icons {
                Text => "bisecting",
                Icons | MinimalIcons => "󰩫 ", //TOOD
            },
            Reverting => match &icons {
                Text => "reverting",
                Icons | MinimalIcons => "",
            },
            CherryPicking => match &icons {
                Text => "cherry-picking",
                Icons | MinimalIcons => "",
            },
            Merging => match &icons {
                Text => "merging",
                Icons | MinimalIcons => "",
            },
            Rebasing => match &icons {
                Text => "rebasing",
                Icons | MinimalIcons => "󰝖",
            },
            ReturnOk => match &icons {
                Text => "OK",
                Icons | MinimalIcons => "✓",
            },
            ReturnFail => match &icons {
                Text => "Failed",
                Icons | MinimalIcons => "✗",
            },
            ReturnNA => match &icons {
                Text => "N/A",
                Icons | MinimalIcons => "⁇",
            },
            TookTime => match &icons {
                Text => "took",
                Icons | MinimalIcons => "",
            },
            Venv => match &icons {
                Text => "py",
                Icons | MinimalIcons => "",
            },
        }
    }
}
