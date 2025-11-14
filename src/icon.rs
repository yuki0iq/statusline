use std::fmt::{Display, Formatter};

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
    fn icon(&self, mode: IconMode) -> &'static str;
}

/// Pretty formatter with respect to selected icon mode
pub trait Pretty {
    /// Pretty formats the object
    fn pretty(&self, f: &mut Formatter<'_>, mode: IconMode) -> std::fmt::Result;
}

pub(crate) fn display(pretty: &dyn Pretty, mode: IconMode) -> impl Display {
    // XXX: Use `std::fmt::from_fn` when 1.93 hits
    struct DisplayHelper<'a>(&'a dyn Pretty, IconMode);
    impl Display for DisplayHelper<'_> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            self.0.pretty(f, self.1)
        }
    }
    DisplayHelper(pretty, mode)
}
