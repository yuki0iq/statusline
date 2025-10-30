use crate::{Environment, Extend, Icon, IconMode, Pretty, Style as _};
use std::path::PathBuf;

pub struct UnseenMail {
    count: usize,
}

impl Extend for UnseenMail {
    fn extend(self: Box<Self>) -> Box<dyn Pretty> {
        self
    }
}

impl UnseenMail {
    pub fn new(environ: &Environment) -> Box<Self> {
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
        Box::new(UnseenMail {
            count: unseen_count + unread_count,
        })
    }
}

impl Pretty for UnseenMail {
    fn pretty(&self, mode: IconMode) -> Option<String> {
        0.ne(&self.count).then(|| {
            format!("[{}{}]", self.icon(mode), self.count)
                .visible()
                .yellow()
                .with_reset()
                .invisible()
                .to_string()
        })
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
