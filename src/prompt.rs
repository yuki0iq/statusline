use crate::chassis::Chassis;
use std::env;

pub enum PromptMode {
    TextMode,
    NerdfontMode { is_minimal: bool },
}

pub struct Prompt {
    mode: PromptMode,
    chassis: Chassis,
}

impl PromptMode {
    pub fn build() -> Self {
        match env::var("PS1_MODE") {
            Ok(x) if x == "text" => PromptMode::TextMode,
            Ok(x) if x == "minimal" => PromptMode::NerdfontMode { is_minimal: true },
            _ => PromptMode::NerdfontMode { is_minimal: false },
        }
    }
}

impl Prompt {
    pub fn build() -> Self {
        Prompt {
            mode: PromptMode::build(),
            chassis: Chassis::get(),
        }
    }

    pub fn host_text(&self) -> &str {
        match &self.mode {
            PromptMode::TextMode => "",
            PromptMode::NerdfontMode { .. } => self.chassis.icon(),
        }
    }

    pub fn user_text(&self) -> &str {
        match &self.mode {
            PromptMode::TextMode => "",
            PromptMode::NerdfontMode { .. } => "",
        }
    }

    pub fn hostuser_at(&self) -> &str {
        match &self.mode {
            PromptMode::TextMode => "@",
            PromptMode::NerdfontMode { .. } => "＠",
        }
    }

    pub fn hostuser_left(&self) -> &str {
        match &self.mode {
            PromptMode::TextMode => "<",
            PromptMode::NerdfontMode { .. } => "[",
        }
    }

    pub fn hostuser_right(&self) -> &str {
        match &self.mode {
            PromptMode::TextMode => ">",
            PromptMode::NerdfontMode { .. } => "]",
        }
    }

    pub fn read_only(&self) -> &str {
        match &self.mode {
            PromptMode::TextMode => "R/O",
            PromptMode::NerdfontMode { .. } => "",
        }
    }

    pub fn on_branch(&self) -> &str {
        match &self.mode {
            PromptMode::TextMode => "on",
            PromptMode::NerdfontMode { .. } => "",
        }
    }

    pub fn at_commit(&self) -> &str {
        match &self.mode {
            PromptMode::TextMode => "at",
            PromptMode::NerdfontMode { .. } => "",
        }
    }

    pub fn ahead(&self) -> &str {
        match &self.mode {
            PromptMode::TextMode => "^",
            PromptMode::NerdfontMode { .. } => "󰞙 ",
        }
    }

    pub fn behind(&self) -> &str {
        match &self.mode {
            PromptMode::TextMode => "v",
            PromptMode::NerdfontMode { .. } => "󰞕 ",
        }
    }

    pub fn stash(&self) -> &str {
        match &self.mode {
            PromptMode::TextMode => "*",
            PromptMode::NerdfontMode { .. } => " ",
        }
    }

    pub fn conflict(&self) -> &str {
        match &self.mode {
            PromptMode::TextMode => "~",
            PromptMode::NerdfontMode { is_minimal: false } => "󰞇 ",
            PromptMode::NerdfontMode { is_minimal: true } => " ",
        }
    }

    pub fn staged(&self) -> &str {
        match &self.mode {
            PromptMode::TextMode => "+",
            PromptMode::NerdfontMode { .. } => " ",
        }
    }

    pub fn dirty(&self) -> &str {
        match &self.mode {
            PromptMode::TextMode => "!",
            PromptMode::NerdfontMode { .. } => " ",
        }
    }

    pub fn untracked(&self) -> &str {
        match &self.mode {
            PromptMode::TextMode => "?",
            PromptMode::NerdfontMode { is_minimal: false } => " ",
            PromptMode::NerdfontMode { is_minimal: true } => " ",
        }
    }

    pub fn return_ok(&self) -> &str {
        match &self.mode {
            PromptMode::TextMode => "OK",
            PromptMode::NerdfontMode { .. } => "✓",
        }
    }

    pub fn return_fail(&self) -> &str {
        match &self.mode {
            PromptMode::TextMode => "Failed",
            PromptMode::NerdfontMode { .. } => "✗",
        }
    }

    pub fn return_unavailable(&self) -> &str {
        match &self.mode {
            PromptMode::TextMode => "N/A",
            PromptMode::NerdfontMode { .. } => "⁇",
        }
    }

    pub fn took_time(&self) -> &str {
        match &self.mode {
            PromptMode::TextMode => "took",
            PromptMode::NerdfontMode { .. } => "",
        }
    }
}
