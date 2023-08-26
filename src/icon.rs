use crate::chassis::Chassis;
use std::env;

/// Icons' modes
pub enum PromptMode {
    /// Use text instead of icons
    TextMode,
    /// Use icons from nerdfonts
    NerdfontMode {
        /// Use alternative icon set (simpler icons, but sometimes hard to get the meaning)
        is_minimal: bool,
    },
}

impl PromptMode {
    /// Detect prompt mode from `PS1_MODE` environment variable
    ///
    /// | Environment        | Resulting PromptMode |
    /// |--------------------|----------------------|
    /// | `PS1_MODE=text`    | Text                 |
    /// | `PS1_MODE=minimal` | Alternative nerdfont |
    /// | otherwise          | Default nerdfont     |
    pub fn build() -> Self {
        match env::var("PS1_MODE") {
            Ok(x) if x == "text" => PromptMode::TextMode,
            Ok(x) if x == "minimal" => PromptMode::NerdfontMode { is_minimal: true },
            _ => PromptMode::NerdfontMode { is_minimal: false },
        }
    }
}

/// Statusline icons getter with respect to [PromptMode] and [Chassis]
///
/// This object is intended to be constructed only once per statusline construction because
/// icon mode and chassis are not fixed and may change suddenly
///
/// TODO: rename? this is cringe why icongetter is prompt
pub struct Prompt {
    mode: PromptMode,
    chassis: Chassis,
}

impl Prompt {
    /// Constructs "prompt" from environment and system info
    pub fn build() -> Self {
        Prompt {
            mode: PromptMode::build(),
            chassis: Chassis::get(),
        }
    }

    pub fn host_text(&self) -> &'static str {
        match &self.mode {
            PromptMode::TextMode => "",
            PromptMode::NerdfontMode { .. } => self.chassis.icon(),
        }
    }

    pub fn user_text(&self) -> &'static str {
        match &self.mode {
            PromptMode::TextMode => "",
            PromptMode::NerdfontMode { .. } => "",
        }
    }

    pub fn hostuser_at(&self) -> &'static str {
        match &self.mode {
            PromptMode::TextMode => "@",
            PromptMode::NerdfontMode { .. } => "＠",
        }
    }

    pub fn hostuser_left(&self) -> &'static str {
        match &self.mode {
            PromptMode::TextMode => "<",
            PromptMode::NerdfontMode { .. } => "[",
        }
    }

    pub fn hostuser_right(&self) -> &'static str {
        match &self.mode {
            PromptMode::TextMode => ">",
            PromptMode::NerdfontMode { .. } => "]",
        }
    }

    pub fn read_only(&self) -> &'static str {
        match &self.mode {
            PromptMode::TextMode => "R/O",
            PromptMode::NerdfontMode { .. } => "",
        }
    }

    pub fn on_branch(&self) -> &'static str {
        match &self.mode {
            PromptMode::TextMode => "on",
            PromptMode::NerdfontMode { .. } => "",
        }
    }

    pub fn at_commit(&self) -> &'static str {
        match &self.mode {
            PromptMode::TextMode => "at",
            PromptMode::NerdfontMode { .. } => "",
        }
    }

    pub fn ahead(&self) -> &'static str {
        match &self.mode {
            PromptMode::TextMode => "^",
            PromptMode::NerdfontMode { .. } => "󰞙 ",
        }
    }

    pub fn behind(&self) -> &'static str {
        match &self.mode {
            PromptMode::TextMode => "v",
            PromptMode::NerdfontMode { .. } => "󰞕 ",
        }
    }

    pub fn stash(&self) -> &'static str {
        match &self.mode {
            PromptMode::TextMode => "*",
            PromptMode::NerdfontMode { .. } => " ",
        }
    }

    pub fn conflict(&self) -> &'static str {
        match &self.mode {
            PromptMode::TextMode => "~",
            PromptMode::NerdfontMode { is_minimal: false } => "󰞇 ",
            PromptMode::NerdfontMode { is_minimal: true } => " ",
        }
    }

    pub fn staged(&self) -> &'static str {
        match &self.mode {
            PromptMode::TextMode => "+",
            PromptMode::NerdfontMode { .. } => " ",
        }
    }

    pub fn dirty(&self) -> &'static str {
        match &self.mode {
            PromptMode::TextMode => "!",
            PromptMode::NerdfontMode { .. } => " ",
        }
    }

    pub fn untracked(&self) -> &'static str {
        match &self.mode {
            PromptMode::TextMode => "?",
            PromptMode::NerdfontMode { is_minimal: false } => " ",
            PromptMode::NerdfontMode { is_minimal: true } => " ",
        }
    }

    pub fn git_bisect(&self) -> &'static str {
        match &self.mode {
            PromptMode::TextMode => "bisecting",
            PromptMode::NerdfontMode { .. } => "󰩫 ", //TOOD
        }
    }

    pub fn git_revert(&self) -> &'static str {
        match &self.mode {
            PromptMode::TextMode => "reverting",
            PromptMode::NerdfontMode { .. } => "",
        }
    }

    pub fn git_cherry(&self) -> &'static str {
        match &self.mode {
            PromptMode::TextMode => "cherry-picking",
            PromptMode::NerdfontMode { .. } => "",
        }
    }

    pub fn git_merge(&self) -> &'static str {
        match &self.mode {
            PromptMode::TextMode => "merging",
            PromptMode::NerdfontMode { .. } => "",
        }
    }

    pub fn git_rebase(&self) -> &'static str {
        match &self.mode {
            PromptMode::TextMode => "rebasing",
            PromptMode::NerdfontMode { .. } => "󰝖",
        }
    }

    pub fn return_ok(&self) -> &'static str {
        match &self.mode {
            PromptMode::TextMode => "OK",
            PromptMode::NerdfontMode { .. } => "✓",
        }
    }

    pub fn return_fail(&self) -> &'static str {
        match &self.mode {
            PromptMode::TextMode => "Failed",
            PromptMode::NerdfontMode { .. } => "✗",
        }
    }

    pub fn return_unavailable(&self) -> &'static str {
        match &self.mode {
            PromptMode::TextMode => "N/A",
            PromptMode::NerdfontMode { .. } => "⁇",
        }
    }

    pub fn took_time(&self) -> &'static str {
        match &self.mode {
            PromptMode::TextMode => "took",
            PromptMode::NerdfontMode { .. } => "",
        }
    }

    pub fn venv(&self) -> &'static str {
        match &self.mode {
            PromptMode::TextMode => "py",
            PromptMode::NerdfontMode { .. } => "",
        }
    }
}
