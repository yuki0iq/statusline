use crate::{Extend, IconMode, Pretty};

pub struct Separator(pub &'static str);

impl Separator {
    pub fn empty() -> Box<Self> {
        Box::new(Self(""))
    }

    pub fn continuation() -> Box<Self> {
        Box::new(Self("\u{f105}"))
    }
}

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
