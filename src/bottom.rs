use crate::{autopretty, BlockType, Environment, FromEnv, Icons, Pretty};

/// The bottom part of statusline. Immutable, intended to use in `readline`-like functions
pub struct Bottom {
    blocks: Vec<Box<dyn Pretty>>,
}

impl FromEnv for Bottom {
    fn from_env(args: &Environment) -> Self {
        Self {
            blocks: vec![
                BlockType::Jobs.create_from_env(args),
                BlockType::ReturnCode.create_from_env(args),
                BlockType::RootShell.create_from_env(args),
            ],
        }
    }
}

impl Pretty for Bottom {
    /// Format the bottom part of the statusline.
    fn pretty(&self, icons: &Icons) -> Option<String> {
        let bottom_line = autopretty(&self.blocks, icons, " ");
        Some(format!("{} ", bottom_line))
    }
}
