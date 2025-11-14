use crate::{Environment, Block, IconMode, Pretty, Style as _};

pub struct Jobs(usize);

impl Block for Jobs {}

impl Jobs {
    pub fn new(args: &Environment) -> Box<dyn Block> {
        Box::new(Jobs(args.jobs_count))
    }
}

impl Pretty for Jobs {
    fn pretty(&self, _: IconMode) -> Option<String> {
        if self.0 == 0 {
            None?;
        }

        let text = if self.0 == 1 { "job" } else { "jobs" };

        Some(
            format!("[{} {text}]", self.0)
                .visible()
                .green()
                .bold()
                .with_reset()
                .invisible()
                .to_string(),
        )
    }
}
