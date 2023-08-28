use crate::{Environment, Icons, Pretty, Style};

pub struct Jobs(usize);

impl From<&Environment> for Jobs {
    fn from(args: &Environment) -> Self {
        Jobs(args.jobs_count)
    }
}

impl Pretty for Jobs {
    fn pretty(&self, _: &Icons) -> Option<String> {
        if self.0 == 0 {
            None?
        }

        let text = if self.0 == 1 { "job" } else { "jobs" };

        Some(
            format!("{} {text}", self.0)
                .boxed()
                .visible()
                .green()
                .bold()
                .with_reset()
                .to_string(),
        )
    }
}
