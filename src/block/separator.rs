use crate::{Block, IconMode, Pretty};

pub struct Separator(pub &'static str);

impl Separator {
    pub fn empty() -> Box<dyn Block> {
        Box::new(Self(""))
    }

    pub fn continuation() -> Box<dyn Block> {
        Box::new(Self("\u{f105}"))
    }
}

impl Pretty for Separator {
    fn pretty(&self, _: IconMode) -> Option<String> {
        Some(self.0.into())
    }
}

impl Block for Separator {}
