use crate::{Block, Environment, IconMode, Pretty};

pub struct Separator;

impl Block for Separator {
    fn new(_: &Environment) -> Option<Box<dyn Block>> {
        Some(Box::new(Self))
    }
}
impl Pretty for Separator {
    fn pretty(&self, _: IconMode) -> Option<String> {
        Some(String::new())
    }
}
