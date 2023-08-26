use crate::Chassis;
use std::env;

/// Icons' modes
pub enum IconMode {
    /// Use text instead of icons
    Text,
    /// Use icons from nerdfonts
    Icons,
    /// Use alternative icon set (simpler icons, but sometimes hard to get the meaning)
    MinimalIcons,
}

impl IconMode {
    /// Detect prompt mode from `PS1_MODE` environment variable
    ///
    /// | Environment        | Resulting IconMode   |
    /// |--------------------|----------------------|
    /// | `PS1_MODE=text`    | Text                 |
    /// | `PS1_MODE=minimal` | Alternative nerdfont |
    /// | otherwise          | Default nerdfont     |
    pub fn build() -> Self {
        match env::var("PS1_MODE") {
            Ok(x) if x == "text" => IconMode::Text,
            Ok(x) if x == "minimal" => IconMode::MinimalIcons,
            _ => IconMode::Icons,
        }
    }
}

impl FnOnce<(Icon,)> for IconMode {
    type Output = &'static str;
    extern "rust-call" fn call_once(self, args: (Icon,)) -> Self::Output {
        args.0.pretty(&self)
    }
}

impl FnMut<(Icon,)> for IconMode {
    extern "rust-call" fn call_mut(&mut self, args: (Icon,)) -> Self::Output {
        args.0.pretty(self)
    }
}

impl Fn<(Icon,)> for IconMode {
    extern "rust-call" fn call(&self, args: (Icon,)) -> Self::Output {
        args.0.pretty(self)
    }
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
    fn pretty(&self, mode: &IconMode) -> &'static str {
        use Icon::*;
        use IconMode::*;
        match &self {
            Host => match &mode {
                Text => "",
                Icons | MinimalIcons => Chassis::get().icon(),
            },
            User => match &mode {
                Text => "as",
                Icons | MinimalIcons => "",
            },
            HostAt => match &mode {
                Text => " at ",
                Icons | MinimalIcons => "＠",
            },
            ReadOnly => match &mode {
                Text => "R/O",
                Icons | MinimalIcons => "",
            },
            OnBranch => match &mode {
                Text => "on",
                Icons | MinimalIcons => "",
            },
            AtCommit => match &mode {
                Text => "at",
                Icons | MinimalIcons => "",
            },
            Ahead => match &mode {
                Text => "^",
                Icons | MinimalIcons => "󰞙 ",
            },
            Behind => match &mode {
                Text => "v",
                Icons | MinimalIcons => "󰞕 ",
            },
            Stashes => match &mode {
                Text => "*",
                Icons | MinimalIcons => " ",
            },
            Conflict => match &mode {
                Text => "~",
                Icons => "󰞇 ",
                MinimalIcons => " ",
            },
            Staged => match &mode {
                Text => "+",
                Icons | MinimalIcons => " ",
            },
            Dirty => match &mode {
                Text => "!",
                Icons | MinimalIcons => " ",
            },
            Untracked => match &mode {
                Text => "?",
                Icons => " ",
                MinimalIcons => " ",
            },
            Bisecting => match &mode {
                Text => "bisecting",
                Icons | MinimalIcons => "󰩫 ", //TOOD
            },
            Reverting => match &mode {
                Text => "reverting",
                Icons | MinimalIcons => "",
            },
            CherryPicking => match &mode {
                Text => "cherry-picking",
                Icons | MinimalIcons => "",
            },
            Merging => match &mode {
                Text => "merging",
                Icons | MinimalIcons => "",
            },
            Rebasing => match &mode {
                Text => "rebasing",
                Icons | MinimalIcons => "󰝖",
            },
            ReturnOk => match &mode {
                Text => "OK",
                Icons | MinimalIcons => "✓",
            },
            ReturnFail => match &mode {
                Text => "Failed",
                Icons | MinimalIcons => "✗",
            },
            ReturnNA => match &mode {
                Text => "N/A",
                Icons | MinimalIcons => "⁇",
            },
            TookTime => match &mode {
                Text => "took",
                Icons | MinimalIcons => "",
            },
            Venv => match &mode {
                Text => "py",
                Icons | MinimalIcons => "",
            },
        }
    }
}
