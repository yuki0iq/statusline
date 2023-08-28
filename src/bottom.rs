use crate::{BlockType, Environment, FromEnv, Icons, Pretty};

/// The bottom part of statusline. Immutable, intended to use in `readline`-like functions
pub struct Bottom {
    blocks: Vec<Box<dyn Pretty>>,
}

impl FromEnv for Bottom {
    fn from_env(args: &Environment) -> Self {
        Self {
            blocks: vec![BlockType::Jobs, BlockType::ReturnCode, BlockType::RootShell]
                .iter()
                .map(|x| x.create_from_env(args))
                .collect(),
        }
    }
}

impl Pretty for Bottom {
    /// Format the bottom part of the statusline.
    fn pretty(&self, icons: &Icons) -> Option<String> {
        let bottom_line = self.blocks.as_slice().pretty(icons)?;
        Some(format!("{} ", bottom_line))
    }
}
