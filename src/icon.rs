use std::env;

/// Icon mode configurer and pretty-printer
///
/// Pretty-prints icons according to selected mode.
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
            Ok(x) if x == "text" => Self::Text,
            Ok(x) if x == "minimal" => Self::MinimalIcons,
            _ => Self::Icons,
        }
    }
}

/// Associated icon getter, which respects icon mode
pub trait Icon {
    /// Returns associated icon with respect to icon mode
    fn icon(&self, mode: &IconMode) -> &'static str;
}

/// Pretty formatter with respect to selected icon mode
pub trait Pretty {
    /// Pretty formats the object
    fn pretty(&self, mode: &IconMode) -> Option<String>;
}

/*
/// All available icons
///
/// Use [Icons] to convert them to string:
#[non_exhaustive]
pub enum _Icon {
    /// Read-only marker
    ReadOnly,
    /// Git info: HEAD is branch
    OnBranch,
    /// Git info: HEAD is a commit
    AtCommit,
    /// Git info: "ahead" the remote
    Ahead,
    /// Git info: "behind" the remote
    Behind,
    /// Git info: stashes
    Stashes,
    /// Git tree: merge conflicts
    Conflict,
    /// Git tree: staged
    Staged,
    /// Git tree: dirty
    Dirty,
    /// Git tree: untracked
    Untracked,
    /// Git action: bisecting
    Bisecting,
    /// Git action: reverting
    Reverting,
    /// Git action: cherry-picking
    CherryPicking,
    /// Git action: merging
    Merging,
    /// Git action: rebasing (with merge backend)
    Rebasing,
    /// Stopwatch icon for last command's execution time
    TookTime,
}

impl Icon {
    fn static_pretty(&self, icons: &Icons) -> &'static str {
        use self::{
            Icon::*,
            Icons::{Icons, MinimalIcons, Text},
        };
        match &self {
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
        }
    }
}
*/
