use crate::{Environment, FromEnv, Icons, Pretty, Style};

pub struct Jobs(usize);

impl FromEnv for Jobs {
    fn from_env(args: &Environment) -> Self {
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
