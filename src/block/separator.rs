use crate::{Extend, IconMode, Pretty};

pub struct Separator(pub &'static str);

impl Pretty for Separator {
    fn pretty(&self, _: &IconMode) -> Option<String> {
        Some(self.0.into())
    }
}

impl Extend for Separator {
    fn extend(self: Box<Self>) -> Box<dyn Pretty> {
        self
    }
}

pub struct Empty;

impl Pretty for Empty {
    fn pretty(&self, _: &IconMode) -> Option<String> {
        None
    }
}

impl Extend for Empty {
    fn extend(self: Box<Self>) -> Box<dyn Pretty> {
        self
    }
}
