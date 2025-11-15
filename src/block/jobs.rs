use crate::{Block, Color, Environment, IconMode, Pretty, Style, WithStyle as _};

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
    fn pretty(&self, f: &mut std::fmt::Formatter<'_>, _: IconMode) -> std::fmt::Result {
        f.with_style(Color::GREEN, Style::BOLD, |f| {
            let text = if self.0 == 1 { "job" } else { "jobs" };
            write!(f, "[{} {text}]", self.0)
        })
    }
}
