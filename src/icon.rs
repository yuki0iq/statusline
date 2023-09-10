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
