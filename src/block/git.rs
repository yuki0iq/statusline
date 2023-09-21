extern crate git2;
use crate::{file, Environment, Icon, IconMode, Pretty, SimpleBlock, Style};
use anyhow::{anyhow, Result};
use std::{
    borrow::Cow,
    fs::{self, File},
    io::{BufRead, BufReader, Error, ErrorKind},
    path::{Path, PathBuf},
};

/*
thanks to
    the git source code which is very fucking clear and understandable
    as well as to purplesyringa's immense help and kind emotional support

thanks to
    https://git-scm.com/docs/git-status
    https://github.com/romkatv/powerlevel10k
    https://docs.rs/git2
    and some more links...

Git Repo status:
  Merging 32df|feature[:master] *3 v1^2
    Merging 32df: current repo state and additional info
    (feature) Current LOCAL branch
    (master) Remote branch IF DIFFERENT and not null
    3 stashes
    1 commit behind, 2 commits ahead

Git Tree status:
  ~4 +5 !6 ?7
    4 unmerged
    5 staged
    6 dirty
    7 untracked
*/

// TODO show tags? annotated ones?
enum Head {
    Branch(String),
    Commit(String),
    Tag(String),
}

impl<'repo> From<git2::Reference<'repo>> for Head {
    fn from(reference: git2::Reference<'repo>) -> Head {
        if reference.is_branch() {
            Head::Branch(reference.name().unwrap_or("refs/heads/<branch>")[11..].to_owned())
        } else if reference.is_tag() {
            Head::Tag(reference.name().unwrap_or("refs/tags/<tag>")[10..].to_owned())
        } else {
            match reference.peel(git2::ObjectType::Commit) {
                Ok(object) => match object.describe(&git2::DescribeOptions::new()) {
                    Ok(describe) => match describe
                        .format(Some(git2::DescribeFormatOptions::new().abbreviated_size(4)))
                    {
                        Ok(abbrev) => Head::Commit(abbrev),
                        Err(_) => Head::Commit("<fmt?>".to_owned()),
                    },
                    Err(_) => Head::Commit("<desc?>".to_owned()),
                },
                Err(_) => Head::Commit("<oid?>".to_owned()),
            }
        }
    }
}

impl Icon for Head {
    fn icon(&self, mode: &IconMode) -> &'static str {
        use IconMode::*;
        match self {
            Self::Branch(_) => match mode {
                Text => "at",
                Icons | MinimalIcons => "",
            },
            Self::Commit(_) => match mode {
                Text => "on",
                Icons | MinimalIcons => "",
            },
            Self::Tag(_) => match mode {
                Text => "tag",
                Icons | MinimalIcons => "TAG", //TODO
            },
        }
    }
}

impl Pretty for Head {
    fn pretty(&self, mode: &IconMode) -> Option<String> {
        let icon = self.icon(mode);
        Some(match &self {
            Head::Branch(name) => format!("{icon} {name}"),
            Head::Commit(id) => format!("{icon} {id}"),
            Head::Tag(tag) => format!("{icon} {tag}"),
        })
    }
}

impl Head {
    // Please WHY
    fn git_value(&self) -> Cow<str> {
        match &self {
            Head::Branch(name) => Cow::from(format!("refs/heads/{name}")),
            Head::Commit(id) => Cow::from(id),
            Head::Tag(name) => Cow::from(format!("refs/tags/{name}")),
        }
    }
}

// TODO: add some info to bisect...
enum State {
    Merging { head: String },
    Rebasing { done: usize, todo: usize },
    CherryPicking { head: String },
    Reverting { head: String },
    Bisecting,
}

impl State {
    fn from_env(root: &Path) -> Option<State> {
        let revert_head = root.join("REVERT_HEAD");
        let cherry_pick_head = root.join("CHERRY_PICK_HEAD");
        let merge_head = root.join("MERGE_HEAD");
        let rebase_merge = root.join("rebase-merge");

        let abbrev_head = |head: &Path| {
            fs::read_to_string(head).map(|mut id| {
                // TODO id.truncate(abbrev_commit(root, &id));
                id.truncate(7);
                id
            })
        };

        Some(if file::exists(&root.join("BISECT_LOG")) {
            State::Bisecting
        } else if let Ok(head) = abbrev_head(&revert_head) {
            State::Reverting { head }
        } else if let Ok(head) = abbrev_head(&cherry_pick_head) {
            State::CherryPicking { head }
        } else if file::exists(&rebase_merge) {
            let todo = match File::open(rebase_merge.join("git-rebase-todo")) {
                Ok(file) => BufReader::new(file)
                    .lines()
                    .map_while(Result::ok)
                    .filter(|line| !line.starts_with('#'))
                    .count(),
                Err(_) => 0,
            };
            let done = match File::open(rebase_merge.join("done")) {
                Ok(file) => BufReader::new(file).lines().count(),
                Err(_) => 0,
            };
            State::Rebasing { todo, done }
        } else if let Ok(head) = abbrev_head(&merge_head) {
            State::Merging { head }
        } else {
            None?
        })
    }
}

impl Icon for State {
    fn icon(&self, mode: &IconMode) -> &'static str {
        use IconMode::*;
        match self {
            Self::Bisecting => match mode {
                Text => "bisecting",
                Icons | MinimalIcons => "󰩫 ", //TOOD
            },
            Self::Reverting { .. } => match mode {
                Text => "reverting",
                Icons | MinimalIcons => "",
            },
            Self::CherryPicking { .. } => match mode {
                Text => "cherry-picking",
                Icons | MinimalIcons => "",
            },
            Self::Merging { .. } => match mode {
                Text => "merging",
                Icons | MinimalIcons => "",
            },
            Self::Rebasing { .. } => match mode {
                Text => "rebasing",
                Icons | MinimalIcons => "󰝖",
            },
        }
    }
}

impl Pretty for State {
    fn pretty(&self, mode: &IconMode) -> Option<String> {
        let icon = self.icon(mode);
        Some(match self {
            State::Bisecting => icon.to_string(),
            State::Reverting { head } => format!("{icon} {}", head),
            State::CherryPicking { head } => {
                format!("{icon} {}", head)
            }
            State::Merging { head } => format!("{icon} {}", head),
            State::Rebasing { done, todo } => {
                format!("{icon} {}/{}", done, done + todo)
            }
        })
    }
}

pub type Repo = Result<RepoInfo>;
pub struct RepoInfo {
    state: Option<State>,
    head: Head,
    remote: Option<String>,
    stashes: usize,
    behind: usize,
    ahead: usize,
}

pub type Tree = Option<TreeImpl>;
pub struct TreeImpl(PathBuf);
pub struct TreeInfo {
    unmerged: usize,
    staged: usize,
    dirty: usize,
    untracked: usize,
}

impl From<&Environment> for Tree {
    fn from(env: &Environment) -> Tree {
        Some(TreeImpl(env.git_tree.as_ref()?.clone()))
    }
}
impl From<&Environment> for Repo {
    fn from(env: &Environment) -> Repo {
        let tree = env
            .git_tree
            .as_ref()
            .ok_or(anyhow!("No git tree found"))?
            .clone();
        let dotgit = tree.join(".git");
        let root = if dotgit.is_file() {
            tree.join(
                fs::read_to_string(&dotgit)?
                    .strip_prefix("gitdir: ")
                    .ok_or(Error::from(ErrorKind::InvalidData))?
                    .trim_end_matches(&['\r', '\n']),
            )
        } else {
            dotgit
        };

        let stash_path = root.join("logs/refs/stash");
        let stashes = fs::File::open(stash_path)
            .map(|file| BufReader::new(file).lines().count())
            .unwrap_or(0);

        let state = State::from_env(&root);

        let repo = git2::Repository::open(tree)?;
        let head = Head::from(repo.head()?);

        let (remote, remote_ref) = if let Head::Branch(_) = head {
            let remote = git2::Branch::wrap(repo.head()?).upstream().ok();
            if let Some(remote) = remote {
                (
                    remote
                        .name()
                        .unwrap_or(None)
                        .and_then(|x| Some(x.split_once('/')?.1))
                        .map(ToOwned::to_owned),
                    Some(remote.into_reference()),
                )
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        let get_ahead_behind = |head_ref: git2::Reference<'_>,
                                remote_ref: Option<git2::Reference<'_>>|
         -> Option<(usize, usize)> {
            repo.graph_ahead_behind(head_ref.target_peel()?, remote_ref?.target_peel()?)
                .ok()
        };
        let (ahead, behind) = get_ahead_behind(repo.head()?, remote_ref).unwrap_or((0, 0));

        Ok(RepoInfo {
            head,
            remote,
            stashes,
            state,
            behind,
            ahead,
        })
    }
}

impl SimpleBlock for Repo {
    fn extend(self: Box<Self>) -> Box<dyn Pretty> {
        self
    }
}

impl SimpleBlock for Tree {
    fn extend(self: Box<Self>) -> Box<dyn Pretty> {
        let tree = match *self {
            Some(x) => x.0,
            _ => return self,
        };

        use git2::Status;
        let unmerged_status = Status::CONFLICTED;
        let staged_status = Status::INDEX_NEW
            | Status::INDEX_DELETED
            | Status::INDEX_RENAMED
            | Status::INDEX_MODIFIED
            | Status::INDEX_TYPECHANGE;
        let dirty_status = Status::WT_DELETED
            | Status::WT_RENAMED
            | Status::WT_MODIFIED
            | Status::WT_TYPECHANGE;
        let untracked_status = Status::WT_NEW;

        let mut unmerged = 0;
        let mut staged = 0;
        let mut dirty = 0;
        let mut untracked = 0;

        let Ok(repo) = git2::Repository::open(tree) else {
            return Box::new(super::separator::Separator("<tree?>"));
        };

        let Ok(statuses) = repo.statuses(Some(git2::StatusOptions::new().include_untracked(true)))
        else {
            return Box::new(super::separator::Separator("<stat?>"));
        };

        for entry in &statuses {
            match entry.status() {
                s if s.intersects(unmerged_status) => unmerged += 1,
                s if s.intersects(staged_status) => staged += 1,
                s if s.intersects(dirty_status) => dirty += 1,
                s if s.intersects(untracked_status) => untracked += 1,
                _ => {}
            }
        }

        Box::new(TreeInfo {
            unmerged,
            staged,
            dirty,
            untracked,
        })
    }
}

impl Pretty for Repo {
    fn pretty(&self, mode: &IconMode) -> Option<String> {
        self.as_ref().ok()?.pretty(mode)
    }
}

impl Pretty for RepoInfo {
    fn pretty(&self, mode: &IconMode) -> Option<String> {
        let mut res = vec![];

        if let Some(state) = &self.state {
            res.push(format!("{}|", state.pretty(mode).unwrap_or_default()));
        }

        let head = self.head.pretty(mode).unwrap_or_default();
        res.push(head);

        match (&self.head, &self.remote) {
            (Head::Branch(local), Some(remote)) if local != remote => {
                res.push(format!(":{}", remote));
            }
            _ => (),
        };

        for (icon, val) in [
            (GitIcon::Stashes, self.stashes),
            (GitIcon::Behind, self.behind),
            (GitIcon::Ahead, self.ahead),
        ] {
            if val != 0 {
                res.push(format!(" {}{}", icon.icon(mode), val));
            }
        }

        Some(
            res.join("")
                .boxed()
                .visible()
                .colorize_with(self.head.git_value().as_ref()) //.pink()
                .bold()
                .with_reset()
                .invisible()
                .to_string(),
        )
    }
}

impl Pretty for Tree {
    fn pretty(&self, _: &IconMode) -> Option<String> {
        None
    }
}

impl Pretty for TreeInfo {
    fn pretty(&self, mode: &IconMode) -> Option<String> {
        let vec = [
            (GitIcon::Conflict, self.unmerged),
            (GitIcon::Staged, self.staged),
            (GitIcon::Dirty, self.dirty),
            (GitIcon::Untracked, self.untracked),
        ]
        .into_iter()
        .filter(|(_, val)| val != &0)
        .map(|(s, val)| format!("{}{}", s.icon(mode), val))
        .collect::<Vec<_>>();

        if vec.is_empty() {
            None
        } else {
            Some(
                vec.join(" ")
                    .boxed()
                    .visible()
                    .pink()
                    .bold()
                    .with_reset()
                    .invisible()
                    .to_string(),
            )
        }
    }
}

enum GitIcon {
    /// Git info: "ahead" the remote
    Ahead,
    /// Git info: "behind" the remote
    Behind,
    /// Git info: stashes
    Stashes,
    /// Git tree: merge conflicts
    Conflict,
    /// Git tree: staged
    Staged,
    /// Git tree: dirty
    Dirty,
    /// Git tree: untracked
    Untracked,
}

impl Icon for GitIcon {
    fn icon(&self, mode: &IconMode) -> &'static str {
        use IconMode::*;
        match &self {
            Self::Ahead => match mode {
                Text => "^",
                Icons | MinimalIcons => "󰞙 ",
            },
            Self::Behind => match mode {
                Text => "v",
                Icons | MinimalIcons => "󰞕 ",
            },
            Self::Stashes => match mode {
                Text => "*",
                Icons | MinimalIcons => " ",
            },
            Self::Conflict => match mode {
                Text => "~",
                Icons => "󰞇 ",
                MinimalIcons => " ",
            },
            Self::Staged => match mode {
                Text => "+",
                Icons | MinimalIcons => " ",
            },
            Self::Dirty => match mode {
                Text => "!",
                Icons | MinimalIcons => " ",
            },
            Self::Untracked => match mode {
                Text => "?",
                Icons => " ",
                MinimalIcons => " ",
            },
        }
    }
}
