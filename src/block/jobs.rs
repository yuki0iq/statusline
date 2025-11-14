use crate::{Block, Environment, IconMode, Pretty, Style as _};

pub struct Jobs(usize);

super::register_block!(Jobs);

impl Block for Jobs {
    fn new(environ: &Environment) -> Option<Box<dyn Block>> {
        if let Some(count) = environ.jobs_count
            && count != 0
        {
            Some(Box::new(Jobs(count)))
        } else {
            None
        }
    }
}

impl Pretty for Jobs {
    fn pretty(&self, _: IconMode) -> Option<String> {
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
