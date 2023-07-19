use dbus::ffidisp::{BusType, Connection};
use std::env;

fn get_chassis() -> Option<String> {
    let conn = Connection::get_private(BusType::System).ok()?;
    let p = conn.with_path(
        "org.freedesktop.hostname1",
        "/org/freedesktop/hostname1",
        5000,
    );
    dbus::ffidisp::stdintf::org_freedesktop_dbus::Properties::get(
        &p,
        "org.freedesktop.hostname1",
        "Chassis",
    )
    .ok()
}

pub enum PromptMode {
    TextMode,
    NerdfontMode,
}

impl PromptMode {
    pub fn new() -> Self {
        if let Ok(val) = env::var("PS1_MODE") && val.to_lowercase() == "text" {
            PromptMode::TextMode
        } else {
            PromptMode::NerdfontMode
        }
    }

    pub fn host_text(&self) -> &str {
        match &self {
            PromptMode::TextMode => "on",
            PromptMode::NerdfontMode => match get_chassis().as_deref() {
                Some("laptop") => "💻",
                Some("desktop") => "🖥 ",
                Some("server") => "🖳",
                Some("tablet") => "具",
                Some("watch") => "⌚️",
                Some("handset") => "🕻",
                Some("vm") => "🖴",
                Some("container") => "☐",
                _ => "󰒋",
            },
        }
    }

    pub fn user_text(&self) -> &str {
        match &self {
            PromptMode::TextMode => "as",
            PromptMode::NerdfontMode => "",
        }
    }

    pub fn read_only(&self) -> &str {
        match &self {
            PromptMode::TextMode => "R/O",
            PromptMode::NerdfontMode => "",
        }
    }
}
