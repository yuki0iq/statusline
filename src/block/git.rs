use crate::{file, Environment, Icon, IconMode, Pretty, SimpleBlock, Style};
use anyhow::{anyhow, bail, Result};
use mmarinus::{perms, Map, Private};
use std::{
    fs::{self, File},
    io::{BufRead, BufReader, Error, ErrorKind},
    iter, mem,
    path::{Path, PathBuf},
    process::Command,
};

/*
thanks to
    the git source code which is very fucking clear and understandable
    as well as to purplesyringa's immense help and kind emotional support

thanks to
    https://git-scm.com/docs/git-status
    https://github.com/romkatv/powerlevel10k
 feature[:master] v1^2 *3 ~4 +5 !6 ?7
    (feature) Current LOCAL branch   -> # branch.head <name>
    (master) Remote branch IF DIFFERENT and not null   -> # branch.upstream <origin>/<name>
    1 commit behind, 2 commits ahead   -> # branch.ab +<ahead> -<behind>
    3 stashes   -> # stash <count>
    4 unmerged   -> XX
    5 staged   -> X.
    6 dirty   -> .X
    7 untracked   -> ?
*/

fn parse_ref_by_name<T: AsRef<str>>(name: T, root: PathBuf) -> Head {
    if let Some(name) = name.as_ref().trim().strip_prefix("refs/heads/") {
        Head {
            kind: HeadKind::Branch(name.to_owned()),
            root,
        }
    } else {
        Head {
            kind: HeadKind::Unknown,
            root,
        }
    }
}

fn lcp<T: AsRef<str>>(a: T, b: T) -> usize {
    iter::zip(a.as_ref().chars(), b.as_ref().chars())
        .position(|(a, b)| a != b)
        .unwrap_or(0) // if equal then LCP should be zero
}

fn lcp_bytes(a: &[u8], b: &[u8]) -> usize {
    let pos = iter::zip(a.iter(), b.iter()).position(|(a, b)| a != b);
    match pos {
        None => 0,
        Some(i) => i * 2 + ((a[i] >> 4) == (b[i] >> 4)) as usize,
    }
}

fn load_objects(root: &Path, fanout: &str) -> Result<Vec<String>> {
    Ok(fs::read_dir(root.join("objects").join(fanout))?
        .map(|res| res.map(|e| String::from(e.file_name().to_string_lossy())))
        .collect::<Result<Vec<_>, _>>()?)
}

fn objects_dir_len(root: &Path, id: &str) -> Result<usize> {
    let (fanout, rest) = id.split_at(2);

    // Find len from ".git/objects/xx/..."
    let best_lcp = load_objects(root, fanout)?
        .iter()
        .map(|val| lcp(val.as_str(), rest))
        .max();
    Ok(match best_lcp {
        None => 2,
        Some(val) => 3 + val,
    })
}

fn packed_objects_len(root: &Path, commit: &str) -> Result<usize> {
    let commit = fahtsex::parse_oid_str(commit).ok_or(Error::from(ErrorKind::InvalidData))?;

    let mut res = 0;
    for entry in fs::read_dir(root.join("objects/pack"))? {
        let path = entry?.path();
        // eprintln!("entry {path:?}");
        let Some(ext) = path.extension() else {
            continue;
        };
        if ext != "idx" {
            continue;
        }

        let map = Map::load(path, Private, perms::Read)?;
        // eprintln!("mmaped");

        // Git packed objects index file format is easy -- Yuki
        // Statements dreamed up by the utterly deranged -- purplesyringa
        // See https://github.com/purplesyringa/gitcenter -> main/dist/js/git.md

        let map_size = map.size() / 4;
        let integers: &[u32] = unsafe { mem::transmute(&map[..4 * map_size]) };

        // Should contain 0x102 ints (magic, version and fanout)
        if map_size < 0x102 {
            continue;
        }

        let (magic, version) = (integers[0], integers[1]);
        let fanout_table: &[u32] = &integers[2..0x102];

        // Magic int is 0xFF744F63 ('\377tOc')
        // probably should be read as "table of contents" which this index is
        // Only version 2 is supported
        if magic != 0xFF744F63 && version != 2 {
            continue;
        }

        // eprintln!("magic + version ok");
        // [0x0008 -- 0x0408] is fanout table as [u32, 256]
        // where `table[i]` is count of objects with `fanout <= i`
        // object range is from `table[i-1]` to `table[i] - 1` including both borders
        let fanout = *commit.first().unwrap() as usize;
        let begin = if fanout == 0 {
            0
        } else {
            fanout_table[fanout - 1]
        } as usize;
        let end = fanout_table[fanout] as usize;

        // begin and end are sha1 *indexes* and not positions of their beginning
        if begin == end {
            continue;
        }

        let commit_position = |idx: usize| 0x102 + 5 * idx;
        if map_size < commit_position(*fanout_table.last().unwrap() as usize) {
            continue;
        }

        let hashes: &[[u8; 20]] =
            unsafe { mem::transmute(&integers[commit_position(begin)..commit_position(end)]) };

        //eprintln!("left and right are {left:?} and {right:?}");

        let index = hashes.partition_point(|hash| hash < &commit);
        if index > 0 {
            res = res.max(lcp_bytes(&hashes[begin + index - 1], &commit));
        }
        if index < end - begin {
            res = res.max(lcp_bytes(
                &hashes[begin + index + (hashes[begin + index] == commit) as usize],
                &commit,
            ));
        }
    }
    //eprintln!("packed: {res:?}");
    //eprintln!("");
    Ok(1 + res)
}

fn abbrev_commit(root: &Path, id: &str) -> usize {
    let mut abbrev_len = 4;
    if let Ok(x) = objects_dir_len(root, id) {
        abbrev_len = abbrev_len.max(x);
    }
    if let Ok(x) = packed_objects_len(root, id) {
        abbrev_len = abbrev_len.max(x);
    }
    abbrev_len
}

enum HeadKind {
    Branch(String),
    Commit(String),
    Unknown,
}

struct Head {
    root: PathBuf,
    kind: HeadKind,
}

impl Icon for HeadKind {
    fn icon(&self, mode: &IconMode) -> &'static str {
        use IconMode::*;
        match self {
            Self::Branch(_) => match mode {
                Text => "at",
                Icons | MinimalIcons => "",
            },
            Self::Commit(_) => match mode {
                Text => "at",
                Icons | MinimalIcons => "",
            },
            Self::Unknown => "<unknown>",
        }
    }
}

impl Pretty for Head {
    fn pretty(&self, mode: &IconMode) -> Option<String> {
        Some(match &self.kind {
            branch @ HeadKind::Branch(name) => format!("{} {}", branch.icon(mode), name),
            oid @ HeadKind::Commit(id) => {
                format!(
                    "{} {}",
                    oid.icon(mode),
                    &id[..abbrev_commit(&self.root, id)]
                )
                // TODO show tag?
            }
            other => other.icon(mode).to_string(),
        })
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
                id.truncate(abbrev_commit(root, &id));
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

fn get_remote(head: &Head) -> Option<(String, String)> {
    let HeadKind::Branch(br) = &head.kind else {
        return None;
    };

    let root = &head.root;
    let section = format!("[branch \"{br}\"]");
    let mut remote_name = None;
    let mut remote_branch = None;
    for line in BufReader::new(fs::File::open(root.join("config")).ok()?)
        .lines()
        .map_while(Result::ok)
        .skip_while(|x| x != &section)
        .skip(1)
        .take_while(|x| x.starts_with('\t'))
    {
        if let Some(x) = line.strip_prefix("\tremote = ") {
            remote_name = Some(x.to_string());
        } else if let Some(x) = line.strip_prefix("\tmerge = refs/heads/") {
            remote_branch = Some(x.to_string());
        }
    }
    remote_name.zip(remote_branch)
}

/// Fast git status information from `.git` folder
pub struct GitStatus {
    tree: PathBuf,
    head: Head,
    remote: Option<(String, String)>,
    stashes: usize,
    state: Option<State>,
}

/// Additional git status information, about branch tracking and working tree state
pub struct GitStatusExtended {
    gs: Box<ResGit>,
    behind: usize,
    ahead: usize,
    unmerged: usize,
    staged: usize,
    dirty: usize,
    untracked: usize,
}

pub type ResGit = Result<GitStatus>;

impl From<&Environment> for ResGit {
    fn from(env: &Environment) -> Result<GitStatus> {
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
        // eprintln!("try find stashes in {stash_path:?}");
        let stashes = fs::File::open(stash_path)
            .map(|file| BufReader::new(file).lines().count())
            .unwrap_or(0);

        let state = State::from_env(&root);

        // eprintln!("ok tree {tree:?} | {root:?}");
        let head_path = root.join("HEAD");

        let head = if head_path.is_symlink() {
            parse_ref_by_name(
                fs::read_link(head_path)?
                    .to_str()
                    .ok_or(Error::from(ErrorKind::InvalidFilename))?,
                root,
            )
        } else {
            let head = fs::read_to_string(head_path)?;
            if let Some(rest) = head.strip_prefix("ref:") {
                parse_ref_by_name(rest, root)
            } else {
                Head {
                    kind: HeadKind::Commit(
                        head.split_whitespace()
                            .next()
                            .unwrap_or_default()
                            .to_owned(),
                    ),
                    root,
                }
            }
        };

        let remote = get_remote(&head);

        Ok(GitStatus {
            tree,
            head,
            remote,
            stashes,
            state,
        })
    }
}

impl SimpleBlock for ResGit {
    fn extend(self: Box<Self>) -> Box<dyn Pretty> {
        let self_ref = match self.as_ref() {
            Result::Ok(a) => a,
            _ => return self,
        };

        let out = Command::new("git")
            .arg("-C")
            .arg(&self_ref.tree)
            .arg("status")
            .arg("--porcelain=2")
            .output()
            .ok();
        let Some(out) = out else {
            return self;
        };
        let lines = out.stdout.split(|&c| c == b'\n').peekable();

        let (ahead, behind) = self_ref.get_ahead_behind().unwrap_or((0, 0));

        // println!("ahead and behind is {ahead} {behind}");

        let mut unmerged = 0;
        let mut staged = 0;
        let mut dirty = 0;
        let mut untracked = 0;

        for line in lines {
            let words: Vec<_> = line.split(|&c| c == b' ').take(2).collect();
            if words.len() != 2 {
                continue;
            }
            let (id, pat) = (words[0], words[1]);
            match (id, pat) {
                (b"?", _) => {
                    untracked += 1;
                }
                (b"u", _) => {
                    unmerged += 1;
                }
                (_, pat) if pat.len() == 2 => {
                    if pat[0] != b'.' {
                        staged += 1;
                    }
                    if pat[1] != b'.' {
                        dirty += 1;
                    }
                }
                _ => {}
            }
        }

        Box::new(GitStatusExtended {
            gs: self,
            behind,
            ahead,
            unmerged,
            staged,
            dirty,
            untracked,
        })
    }
}

impl GitStatus {
    fn get_ahead_behind(&self) -> Result<(usize, usize)> {
        let (HeadKind::Branch(head), Some((name, branch))) = (&self.head.kind, &self.remote) else {
            bail!("Head is not a branch or remote is missing");
        };
        Ok(Command::new("git")
            .arg("-C")
            .arg(&self.tree)
            .arg("rev-list")
            .arg("--count")
            .arg("--left-right")
            .arg(format!("{head}...{name}/{branch}"))
            .output()?
            .stdout
            .trim_ascii_end()
            .split(|&c| c == b'\t')
            .map(|x| Result::<usize>::Ok(std::str::from_utf8(x)?.parse::<usize>()?))
            .filter_map(Result::ok)
            .next_chunk::<2>()
            .map_err(|_| anyhow!("Invalid rev-list output"))?
            .into())
    }
}

impl Pretty for ResGit {
    fn pretty(&self, mode: &IconMode) -> Option<String> {
        let ans = self.as_ref().ok()?.pretty(mode)?;
        Some(
            format!("{ans}...")
                .boxed()
                .pink()
                .bold()
                .with_reset()
                .to_string(),
        )
    }
}

impl Pretty for GitStatus {
    fn pretty(&self, mode: &IconMode) -> Option<String> {
        let mut res = vec![];

        if let Some(state) = &self.state {
            res.push(format!("{}|", state.pretty(mode).unwrap_or_default()));
        }

        let head = self.head.pretty(mode).unwrap_or_default();
        res.push(head);

        match (&self.head.kind, &self.remote) {
            (HeadKind::Branch(head), Some((_, remote))) if head != remote => {
                res.push(format!(":{}", remote));
            }
            _ => (),
        };

        for (s, val) in [(GitIcon::Stashes, self.stashes)] {
            if val != 0 {
                res.push(format!(" {}{}", s.icon(mode), val));
            }
        }

        Some(res.join(""))
    }
}

impl Pretty for GitStatusExtended {
    fn pretty(&self, mode: &IconMode) -> Option<String> {
        Some(
            (self.gs.as_ref().as_ref().unwrap().pretty(mode)?
                + &[
                    (GitIcon::Behind, self.behind),
                    (GitIcon::Ahead, self.ahead),
                    (GitIcon::Conflict, self.unmerged),
                    (GitIcon::Staged, self.staged),
                    (GitIcon::Dirty, self.dirty),
                    (GitIcon::Untracked, self.untracked),
                ]
                .into_iter()
                .filter(|(_, val)| val != &0)
                .map(|(s, val)| format!(" {}{}", s.icon(mode), val))
                .collect::<Vec<_>>()
                .join(""))
                .boxed()
                .pink()
                .bold()
                .with_reset()
                .to_string(),
        )
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
