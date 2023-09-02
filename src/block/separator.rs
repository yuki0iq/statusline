use crate::{IconMode, Pretty, SimpleBlock};

pub struct Separator();

impl Pretty for Separator {
    fn pretty(&self, _: &IconMode) -> Option<String> {
        Some(String::new())
    }
}

impl SimpleBlock for Separator {
    fn extend(self: Box<Self>) -> Box<dyn Pretty> {
        self
    }
}
