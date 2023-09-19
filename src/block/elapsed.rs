use crate::{time, Environment, Icon, IconMode, Pretty, SimpleBlock, Style};

pub struct Elapsed(u64);

impl From<&Environment> for Elapsed {
    fn from(env: &Environment) -> Self {
        Elapsed(env.elapsed_time.unwrap_or_default())
    }
}

impl SimpleBlock for Elapsed {
    fn extend(self: Box<Self>) -> Box<dyn Pretty> {
        self
    }
}

impl Icon for Elapsed {
    fn icon(&self, mode: &IconMode) -> &'static str {
        use IconMode::*;
        match &mode {
            Text => "took",
            Icons | MinimalIcons => "",
        }
    }
}

impl Pretty for Elapsed {
    fn pretty(&self, mode: &IconMode) -> Option<String> {
        Some(
            format!(
                "{} {}",
                self.icon(mode),
                time::microseconds_to_string(self.0)?
            )
            .rounded()
            .visible()
            .cyan()
            .with_reset()
            .invisible()
            .to_string(),
        )
    }
}
