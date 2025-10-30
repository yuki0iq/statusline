use crate::{Extend, IconMode, Pretty};

pub struct Separator(pub &'static str);

impl Pretty for Separator {
    fn pretty(&self, _: IconMode) -> Option<String> {
        Some(self.0.into())
    }
}

impl Extend for Separator {
    fn extend(self: Box<Self>) -> Box<dyn Pretty> {
        self
    }
}
