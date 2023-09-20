use crate::{IconMode, Pretty, SimpleBlock};

pub struct Separator(pub &'static str);

impl Pretty for Separator {
    fn pretty(&self, _: &IconMode) -> Option<String> {
        Some(self.0.to_string())
    }
}

impl SimpleBlock for Separator {
    fn extend(self: Box<Self>) -> Box<dyn Pretty> {
        self
    }
}

pub struct Empty();

impl Pretty for Empty {
    fn pretty(&self, _: &IconMode) -> Option<String> {
        None
    }
}

impl SimpleBlock for Empty {
    fn extend(self: Box<Self>) -> Box<dyn Pretty> {
        self
    }
}
