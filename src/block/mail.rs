use crate::{Environment, Icon, IconMode, Pretty, SimpleBlock, Style};
use std::{env, fs, path::PathBuf};

pub struct UnseenMail {
    count: usize,
}

impl SimpleBlock for UnseenMail {
    fn extend(self: Box<Self>) -> Box<dyn Pretty> {
        self
    }
}

impl From<&Environment> for UnseenMail {
    fn from(environ: &Environment) -> Self {
        let maildir_path =
            env::var("MAIL").unwrap_or_else(|_| format!("/var/spool/mail/{}", environ.user));
        let maildir = PathBuf::from(maildir_path);
        let unseen_count = fs::read_dir(maildir.join("new"))
            .map(Iterator::count)
            .unwrap_or(0);
        let unread_count = fs::read_dir(maildir.join("cur"))
            .map(|iter| {
                iter.map_while(Result::ok)
                    .filter_map(|entry| entry.file_name().into_string().ok())
                    .filter(|filename| filename.ends_with(":2,"))
                    .count()
            })
            .unwrap_or(0);
        UnseenMail {
            count: unseen_count + unread_count,
        }
    }
}

impl Pretty for UnseenMail {
    fn pretty(&self, icons: &IconMode) -> Option<String> {
        0.ne(&self.count).then(|| {
            format!("{}{}", self.icon(icons), self.count)
                .boxed()
                .visible()
                .yellow()
                .with_reset()
                .invisible()
                .to_string()
        })
    }
}

impl Icon for UnseenMail {
    fn icon(&self, mode: &IconMode) -> &'static str {
        use IconMode::*;
        match &mode {
            Text => "eml ",
            Icons | MinimalIcons => "󰇰 ",
        }
    }
}
