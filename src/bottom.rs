use crate::{CommandLineArgs, Icon, Icons, Pretty, Style};
use nix::unistd;

fn autojoin(vec: &[&str], sep: &str) -> String {
    vec.iter()
        .copied()
        .filter(|el| !el.is_empty())
        .collect::<Vec<&str>>()
        .join(sep)
}

/// The bottom part of statusline. Immutable, intended to use in `readline`-like functions
pub struct Bottom {
    is_root: bool,

    // Background jobs count
    jobs: usize,

    // Process return code
    return_code: Option<u8>,
}

impl Bottom {
    pub fn from_env(args: &CommandLineArgs) -> Self {
        Self {
            is_root: unistd::getuid().is_root(),
            jobs: args.jobs_count,
            return_code: args.ret_code,
        }
    }
}

impl Pretty for Bottom {
    /// Format the bottom part of the statusline.
    fn pretty(&self, icons: &Icons) -> String {
        let root = self
            .is_root
            .then_some("#".visible().red())
            .unwrap_or("$".visible().green())
            .bold()
            .with_reset()
            .invisible()
            .to_string();

        let (ok, fail, na) = (
            icons(Icon::ReturnOk).visible(),
            icons(Icon::ReturnFail).visible(),
            icons(Icon::ReturnNA).visible(),
        );
        let returned = match &self.return_code {
            Some(0) | Some(130) => ok.light_green(),
            Some(_) => fail.light_red(),
            None => na.light_gray(),
        }
        .with_reset()
        .invisible()
        .to_string();

        let jobs = 0
            .ne(&self.jobs)
            .then_some(
                format!(
                    "{} job{}",
                    self.jobs,
                    1.ne(&self.jobs).then_some("s").unwrap_or_default()
                )
                .boxed()
                .visible()
                .green()
                .bold()
                .with_reset()
                .invisible()
                .to_string(),
            )
            .unwrap_or_default();

        let bottom_line = autojoin(&[&jobs, &returned, &root], " ");

        format!("{} ", bottom_line)
    }
}
