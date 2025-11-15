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
        let unseen_count = std::fs::read_dir(maildir.join("new"))
            .map(Iterator::count)
            .unwrap_or(0);
        let unread_count = std::fs::read_dir(maildir.join("cur"))
            .map(|iter| {
                iter.map_while(Result::ok)
                    .filter_map(|entry| entry.file_name().into_string().ok())
                    .filter(|filename| filename.ends_with(":2,"))
                    .count()
            })
            .unwrap_or(0);
        let count = unseen_count + unread_count;
        (count > 0).then_some(UnseenMail { count })
    }
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
