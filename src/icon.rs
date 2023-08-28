use crate::Chassis;
use std::env;

/// Icon mode configurer and pretty-printer
///
/// Pretty-prints icons according to selected mode.
///
/// # Example
///
/// ```
/// use statusline::{Icon, Icons};
///
/// let icons = Icons::Text;
/// assert_eq!("R/O", icons(Icon::ReadOnly));
/// ```
///
/// This, however, requires nightly rust and some feature flags.
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

/// Pretty formatter with respect to selected icon mode
pub trait Pretty {
    /// Pretty formats the object
    fn pretty(&self, icons: &Icons) -> Option<String>;
}

impl Pretty for &[Box<dyn Pretty>] {
    fn pretty(&self, icons: &Icons) -> Option<String> {
        // TODO collect -- why??
        Some(
            self.iter()
                .filter_map(|x| x.as_ref().pretty(icons))
                .collect::<Vec<_>>()
                .join(" "),
        )
    }
}

/// All available icons
///
/// Use [Icons] to convert them to string:
#[non_exhaustive]
pub enum Icon {
    /// Chassis icon (near hostname)
    Host,
    /// User icon (near username)
    User,
    /// "At" symbol between hostname and username
    HostAt,
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
    /// Return code is a success
    ReturnOk,
    /// Return code is a failure
    ReturnFail,
    /// Return code information is unavailable (WHY is it an icon WHY do we need a placeholder there)
    ReturnNA,
    /// Stopwatch icon for last command's execution time
    TookTime,
    /// Python logo
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
