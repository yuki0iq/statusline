use crate::{Block, Color, Environment, Icon, IconMode, Pretty, Style, WithStyle as _};
use std::path::PathBuf;

pub struct UnseenMail {
    count: usize,
}

super::register_block!(UnseenMail);

impl Block for UnseenMail {
    fn new(environ: &Environment) -> Option<Self> {
        let maildir_path =
            std::env::var("MAIL").unwrap_or_else(|_| format!("/var/spool/mail/{}", environ.user));
        let maildir = PathBuf::from(maildir_path);
        let unseen_count = ignore_errors(maildir.join("new").read_dir()).count();
        let unread_count = ignore_errors(maildir.join("cur").read_dir())
            .filter(|entry| {
                entry
                    .file_name()
                    .to_str()
                    .unwrap_or_default()
                    .ends_with(":2,")
            })
            .count();
        let count = unseen_count + unread_count;
        (count > 0).then_some(UnseenMail { count })
    }
}

fn ignore_errors<T, E>(
    res: Result<impl Iterator<Item = Result<T, E>>, E>,
) -> impl Iterator<Item = T> {
    res.into_iter().flatten().flatten()
}

impl Pretty for UnseenMail {
    fn pretty(&self, f: &mut std::fmt::Formatter<'_>, mode: IconMode) -> std::fmt::Result {
        f.with_style(Color::YELLOW, Style::empty(), |f| {
            write!(f, "[{} {}]", self.icon(mode), self.count)
        })
    }
}

impl Icon for UnseenMail {
    fn icon(&self, mode: IconMode) -> &'static str {
        use IconMode::*;
        match &mode {
            Text => "eml",
            Icons | MinimalIcons => "󰇰",
        }
    }
}
