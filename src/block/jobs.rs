use crate::{Block, Environment, IconMode, Pretty, Style as _};

pub struct Jobs(usize);

impl Block for Jobs {
    fn new(environ: &Environment) -> Option<Box<dyn Block>> {
        let count = environ.jobs_count;
        if count == 0 {
            None
        } else {
            Some(Box::new(Jobs(count)))
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
