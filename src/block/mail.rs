use crate::{Block, Environment, Icon, IconMode, Pretty, Style as _};
use std::path::PathBuf;

pub struct UnseenMail {
    count: usize,
}

impl Block for UnseenMail {
    fn new(environ: &Environment) -> Option<Box<dyn Block>> {
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
        if count == 0 {
            None
        } else {
            Some(Box::new(UnseenMail { count }))
        }
    }
}

impl Pretty for UnseenMail {
    fn pretty(&self, mode: IconMode) -> Option<String> {
        Some(
            format!("[{}{}]", self.icon(mode), self.count)
                .visible()
                .yellow()
                .with_reset()
                .invisible()
                .to_string(),
        )
    }
}

impl Icon for UnseenMail {
    fn icon(&self, mode: IconMode) -> &'static str {
        use IconMode::*;
        match &mode {
            Text => "eml ",
            Icons | MinimalIcons => "󰇰 ",
        }
    }
}
