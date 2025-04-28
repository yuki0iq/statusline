/// Icon mode configurer
#[non_exhaustive]
#[derive(Clone, Copy)]
pub enum IconMode {
    /// Use text instead of icons
    Text,
    /// Use icons from nerdfonts
    Icons,
    /// Use alternative icon set (simpler icons, but sometimes hard to get the meaning)
    MinimalIcons,
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
