use crate::chassis::Chassis;
use std::env;

pub enum PromptMode {
    TextMode,
    NerdfontMode,
}

pub struct Prompt {
    mode: PromptMode,
    chassis: Chassis,
}

impl PromptMode {
    pub fn build() -> Self {
        if let Ok(val) = env::var("PS1_MODE") && val.to_lowercase() == "text" {
            PromptMode::TextMode
        } else {
            PromptMode::NerdfontMode
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
            PromptMode::NerdfontMode => self.chassis.icon(),
        }
    }

    pub fn user_text(&self) -> &str {
        match &self.mode {
            PromptMode::TextMode => "",
            PromptMode::NerdfontMode => "",
        }
    }

    pub fn hostuser_at(&self) -> &str {
        match &self.mode {
            PromptMode::TextMode => "@",
            PromptMode::NerdfontMode => "＠",
        }
    }

    pub fn hostuser_left(&self) -> &str {
        match &self.mode {
            PromptMode::TextMode => "<",
            PromptMode::NerdfontMode => "[",
        }
    }

    pub fn hostuser_right(&self) -> &str {
        match &self.mode {
            PromptMode::TextMode => ">",
            PromptMode::NerdfontMode => "]",
        }
    }

    pub fn read_only(&self) -> &str {
        match &self.mode {
            PromptMode::TextMode => "R/O",
            PromptMode::NerdfontMode => "",
        }
    }

    pub fn on_branch(&self) -> &str {
        match &self.mode {
            PromptMode::TextMode => "on",
            PromptMode::NerdfontMode => "",
        }
    }

    pub fn return_ok(&self) -> &str {
        match &self.mode {
            PromptMode::TextMode => "OK",
            PromptMode::NerdfontMode => "✓",
        }
    }

    pub fn return_fail(&self) -> &str {
        match &self.mode {
            PromptMode::TextMode => "Failed",
            PromptMode::NerdfontMode => "✗",
        }
    }

    pub fn return_unavailable(&self) -> &str {
        match &self.mode {
            PromptMode::TextMode => "N/A",
            PromptMode::NerdfontMode => "⁇",
        }
    }

    pub fn took_time(&self) -> &str {
        match &self.mode {
            PromptMode::TextMode => "took",
            PromptMode::NerdfontMode => "",
        }
    }
}
