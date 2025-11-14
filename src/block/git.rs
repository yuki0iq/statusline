use crate::{Block, Environment, Icon, IconMode, Pretty, Style as _, file};
use anyhow::{Context as _, Result};
use memmap2::Mmap;
use rustix::process::Signal;
use std::{
    borrow::Cow,
    fs::File,
    io::{BufRead as _, BufReader, Error, ErrorKind, Result as IoResult},
    os::unix::process::CommandExt as _,
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
    let kind = if let Some(name) = name.as_ref().trim().strip_prefix("refs/heads/") {
        HeadKind::Branch(name.to_owned())
    } else {
        HeadKind::Unknown
    };
    Head { root, kind }
}

fn lcp<T: AsRef<str>>(left: T, right: T) -> Option<usize> {
    std::iter::zip(left.as_ref().chars(), right.as_ref().chars()).position(|(a, b)| a != b)
}

fn lcp_bytes(left: &[u8], right: &[u8]) -> Option<usize> {
    let i = std::iter::zip(left.iter(), right.iter()).position(|(a, b)| a != b)?;
    Some(i * 2 + usize::from((left[i] >> 4) == (right[i] >> 4)))
}

fn load_objects(root: &Path, fanout: &str) -> Result<Vec<String>> {
    Ok(std::fs::read_dir(root.join("objects").join(fanout))?
        .map(|res| res.map(|e| String::from(e.file_name().to_string_lossy())))
        .collect::<Result<Vec<_>, _>>()?)
}

fn objects_dir_len(root: &Path, id: &str) -> Result<usize> {
    let (fanout, rest) = id.split_at(2);

    // Find len from ".git/objects/xx/..."
    let best_lcp = load_objects(root, fanout)?
        .iter()
        .filter_map(|val| lcp(val.as_str(), rest))
        .max();
    Ok(match best_lcp {
        None => 2,
        Some(val) => 3 + val,
    })
}

fn parse_oid_slow(hex: &[u8; 40]) -> [u8; 20] {
    fn val(mut x: u8) -> u8 {
        x -= b'0';
        if x >= 10 {
            x -= b'a' - (b'9' + 1);
        }
        x
    }

    let mut result = [0; 20];
    for i in 0..20 {
        result[i] = val(hex[2 * i]) << 4_i32 | val(hex[2 * i + 1]);
    }
    result
}

fn parse_oid_str(hex: &str) -> Option<[u8; 20]> {
    Some(parse_oid_slow(hex.as_bytes().try_into().ok()?))
}

fn packed_objects_len(root: &Path, commit: &str) -> Result<usize> {
    let commit = parse_oid_str(commit).ok_or(Error::from(ErrorKind::InvalidData))?;

    let mut res = 0;
    for entry in std::fs::read_dir(root.join("objects/pack"))? {
        let path = entry?.path();
        // eprintln!("entry {path:?}");
        let Some(ext) = path.extension() else {
            continue;
        };
        if ext != "idx" {
            continue;
        }

        // File should at least contain magic, version and a fanout table, which is 102 ints
        let file = File::open(path).context("open packed objects")?;
        // SAFETY: the packed file should not be edited by anyone
        // XXX: This may not be true, check git sources
        let map_object = unsafe { Mmap::map(&file).context("map packed objects")? };
        let map = &*map_object;
        if map.len() < 0x408 {
            continue;
        }
        // eprintln!("mmaped");

        // Git packed objects index file format is easy -- Yuki
        // Statements dreamed up by the utterly deranged -- purplesyringa
        // See https://github.com/purplesyringa/gitcenter -> main/dist/js/git.md
        //
        // Actually, I don't think this file format is easy. It's easy to make a great lot of bugs
        // in code dealing with this file format. Why did I return here? Because of one fucking
        // small optimization which thought that every i32 is in correct byte order -- the statement
        // which is wrong, but left unnoticed for a long time. -- Yuki, some months later
        //
        // I'd like to never return here again.
        //
        // A year has passed. Holy shit. UB, in Alisa's (probably) code. Lmao

        let map_size = map.len() / 4;
        // SAFETY: 4 * map_size <= map.len()
        let integers: &[[u8; 4]] =
            unsafe { std::slice::from_raw_parts(map.as_ptr().cast(), map_size) };

        // Magic int is 0xFF744F63 ('\377tOc') which probably should be read as "table of contents"
        // Only version 2 is supported
        let magic_version = &map[..8]; // it's okay promise me
        if magic_version != [0xff, 0x74, 0x4f, 0x63, 0x00, 0x00, 0x00, 0x02] {
            continue;
        }

        // [0x0008 -- 0x0408] is fanout table as [u32, 256], but in fucking network byte order
        // where `table[i]` is count of objects with `fanout <= i`
        // object range is from `table[i-1]` to `table[i] - 1` including both borders
        let fanout_table: &[[u8; 4]] = &integers[2..0x102];
        let fanout = *commit.first().unwrap() as usize;
        let begin = if fanout == 0 {
            0
        } else {
            u32::from_be_bytes(fanout_table[fanout - 1])
        } as usize;
        let end = u32::from_be_bytes(fanout_table[fanout]) as usize;

        // eprintln!("left and right are {begin:x?} and {end:x?}");

        // begin and end are sha1 *indexes* and not positions of their beginning
        if begin == end {
            continue;
        }

        let commit_position = |idx: usize| 0x102 + 5 * idx;
        if map_size < commit_position(u32::from_be_bytes(*fanout_table.last().unwrap()) as usize) {
            continue;
        }

        // holy hell, second memory transmute
        let hashes_start =
            // SAFETY: begin = fanout_table[..] <= fanout_table.last() <= map_size
            unsafe { integers.as_ptr().offset(commit_position(begin).cast_signed()).cast() };
        // SAFETY: begin <= end <= map_size
        let hashes: &[[u8; 20]] = unsafe { std::slice::from_raw_parts(hashes_start, end - begin) };

        let index = hashes.partition_point(|hash| hash < &commit);
        // eprintln!("got index {index}");
        if index > 0 {
            res = res.max(lcp_bytes(&hashes[index - 1], &commit).unwrap());
        }
        // Skip hashes[index] if it is an exact match
        let index = index + usize::from(hashes[index] == commit);
        if index < end - begin {
            res = res.max(lcp_bytes(&hashes[index], &commit).unwrap());
        }
    }
    // eprintln!("packed: {res:?}");
    // eprintln!("");
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

#[derive(Debug)]
enum HeadKind {
    Branch(String),
    Unborn(String),
    Commit(String),
    Unknown,
}

#[derive(Debug)]
struct Head {
    root: PathBuf,
    kind: HeadKind,
}

impl Icon for HeadKind {
    fn icon(&self, mode: IconMode) -> &'static str {
        use IconMode::*;
        match self {
            Self::Branch(_) => match mode {
                Text => "on",
                Icons | MinimalIcons => "󰘬",
            },
            Self::Unborn(_) => match mode {
                Text => "to",
                Icons | MinimalIcons => "󰽤",
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
    fn pretty(&self, mode: IconMode) -> Option<String> {
        Some(match &self.kind {
            branch @ (HeadKind::Branch(name) | HeadKind::Unborn(name)) => {
                format!("{} {}", branch.icon(mode), name)
            }
            oid @ HeadKind::Commit(id) => {
                format!(
                    "{} {}",
                    oid.icon(mode),
                    &id[..abbrev_commit(&self.root, id)]
                )
                // TODO show tag?
            }
            other => other.icon(mode).into(),
        })
    }
}

fn is_ref_existing(root: &Path, ref_name: &str) -> bool {
    file::exists(&root.join(ref_name))
        || File::open(root.join("packed-refs"))
            .ok()
            .map(BufReader::new)
            .map(BufReader::lines)
            .map(|lines| lines.map_while(Result::ok))
            .and_then(|mut lines| lines.find(|line| line.contains(ref_name)))
            .is_some()
}

impl Head {
    // Please WHY
    fn ref_name(&self) -> Cow<'_, str> {
        match &self.kind {
            HeadKind::Branch(name) | HeadKind::Unborn(name) => {
                Cow::from(format!("refs/heads/{name}"))
            }
            HeadKind::Commit(id) => Cow::from(id),
            HeadKind::Unknown => Cow::from("<head>"),
        }
    }

    // WHY WHY WHY send help
    fn refine_unborn(mut self) -> Self {
        let root = &self.root;
        self.kind = match self.kind {
            HeadKind::Branch(name) if !is_ref_existing(root, &self.ref_name()) => {
                HeadKind::Unborn(name)
            }

            _ => self.kind,
        };
        self
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
    fn discover(root: &Path) -> Option<State> {
        let revert_head = root.join("REVERT_HEAD");
        let cherry_pick_head = root.join("CHERRY_PICK_HEAD");
        let merge_head = root.join("MERGE_HEAD");
        let rebase_merge = root.join("rebase-merge");

        let abbrev_head = |head: &Path| {
            std::fs::read_to_string(head).map(|mut id| {
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
    fn icon(&self, mode: IconMode) -> &'static str {
        use IconMode::*;
        match self {
            Self::Bisecting => match mode {
                Text => "bi",
                Icons | MinimalIcons => "󰩫 ", //TODO
            },
            Self::Reverting { .. } => match mode {
                Text => "rv",
                Icons | MinimalIcons => "",
            },
            Self::CherryPicking { .. } => match mode {
                Text => "cp",
                Icons | MinimalIcons => "",
            },
            Self::Merging { .. } => match mode {
                Text => "me",
                Icons | MinimalIcons => "󰃸",
            },
            Self::Rebasing { .. } => match mode {
                Text => "rb",
                Icons | MinimalIcons => "󰝖",
            },
        }
    }
}

impl Pretty for State {
    fn pretty(&self, mode: IconMode) -> Option<String> {
        let icon = self.icon(mode);
        Some(match self {
            State::Bisecting => icon.into(),
            State::CherryPicking { head } | State::Reverting { head } | State::Merging { head } => {
                format!("{icon} {head}")
            }
            State::Rebasing { done, todo } => {
                format!("{icon} {}/{}", done, done + todo)
            }
        })
    }
}

struct Remote {
    name: String,
    branch: String,
    exists: bool,
}

fn get_remote(head: &Head) -> Option<Remote> {
    let HeadKind::Branch(br) = &head.kind else {
        return None;
    };

    let root = &head.root;
    let section = format!("[branch \"{br}\"]");
    let mut name = None;
    let mut branch = None;
    for line in BufReader::new(File::open(root.join("config")).ok()?)
        .lines()
        .map_while(Result::ok)
        .skip_while(|x| x != &section)
        .skip(1)
        .take_while(|x| x.starts_with('\t'))
    {
        if let Some(x) = line.strip_prefix("\tremote = ") {
            name = Some(x.into());
        } else if let Some(x) = line.strip_prefix("\tmerge = refs/heads/") {
            branch = Some(x.into());
        }
    }
    let (name, branch) = (name?, branch?);
    let exists = is_ref_existing(root, &format!("refs/remotes/{name}/{branch}"));

    Some(Remote {
        name,
        branch,
        exists,
    })
}

fn get_ahead_behind(
    tree: &Path,
    head: &HeadKind,
    remote: Option<&Remote>,
) -> Option<(usize, usize)> {
    let (
        HeadKind::Branch(head),
        Some(Remote {
            name,
            branch,
            exists: true,
        }),
    ) = (head, remote)
    else {
        return None;
    };

    // This should not be that slow
    let output = Command::new("git")
        .arg("-C")
        .arg(tree)
        .arg("rev-list")
        .arg("--count")
        .arg("--left-right")
        .arg(format!("{head}...{name}/{branch}"))
        .output()
        .ok()?;
    let mut iter = output
        .stdout
        .trim_ascii_end()
        .split(|&c| c == b'\t')
        .flat_map(std::str::from_utf8)
        .flat_map(str::parse::<usize>);
    let ahead = iter.next();
    let behind = iter.next();
    ahead.zip(behind)
}

pub struct GitRepo {
    head: Head,
    remote: Option<Remote>,
    stashes: usize,
    state: Option<State>,
    behind: usize,
    ahead: usize,
}

impl Block for GitRepo {
    fn new(environ: &Environment) -> Option<Box<dyn Block>> {
        let tree = environ.git_tree.as_ref()?.clone();
        let dotgit = tree.join(".git");
        let root = if dotgit.is_file() {
            tree.join(
                std::fs::read_to_string(&dotgit)
                    .ok()?
                    .strip_prefix("gitdir: ")?
                    .trim_end_matches(['\r', '\n']),
            )
        } else {
            dotgit
        };

        let stash_path = root.join("logs/refs/stash");
        // eprintln!("try find stashes in {stash_path:?}");
        let stashes = File::open(stash_path)
            .map(|file| BufReader::new(file).lines().count())
            .unwrap_or(0);

        let state = State::discover(&root);

        // eprintln!("ok tree {tree:?} | {root:?}");
        let head_path = root.join("HEAD");

        let head = if head_path.is_symlink() {
            parse_ref_by_name(std::fs::read_link(head_path).ok()?.to_str()?, root)
        } else {
            let head = std::fs::read_to_string(head_path).ok()?;
            if let Some(rest) = head.strip_prefix("ref:") {
                parse_ref_by_name(rest, root)
            } else {
                let kind = HeadKind::Commit(head.split_whitespace().next()?.to_owned());
                Head { root, kind }
            }
        };
        let head = head.refine_unborn();

        let remote = get_remote(&head);

        let (ahead, behind) =
            get_ahead_behind(&tree, &head.kind, remote.as_ref()).unwrap_or((0, 0));

        Some(Box::new(GitRepo {
            head,
            remote,
            stashes,
            state,
            behind,
            ahead,
        }))
    }
}

pub struct GitTree {
    tree: PathBuf,
    unmerged: usize,
    staged: usize,
    dirty: usize,
    untracked: usize,
}

impl Block for GitTree {
    fn new(environ: &Environment) -> Option<Box<dyn Block>> {
        let tree = environ.git_tree.as_ref()?.clone();
        Some(Box::new(GitTree {
            tree,
            unmerged: 0,
            staged: 0,
            dirty: 0,
            untracked: 0,
        }))
    }

    fn extend(&mut self) {
        let parent_pid = rustix::process::getpid();
        // SAFETY: pre_exec only sets parent process death signal and does nothing more
        let out = unsafe {
            Command::new("git")
                .arg("-C")
                .arg(&self.tree)
                .arg("status")
                .arg("--porcelain=2")
                .pre_exec(move || -> IoResult<()> {
                    rustix::process::set_parent_process_death_signal(Some(Signal::TERM))?;
                    if Some(parent_pid) != rustix::process::getppid() {
                        return Err(Error::other("Parent already dead"));
                    }
                    Ok(())
                })
                .output()
                .ok()
        };
        let Some(out) = out else { return };
        let lines = out.stdout.split(|&c| c == b'\n');

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

        self.unmerged = unmerged;
        self.staged = staged;
        self.dirty = dirty;
        self.untracked = untracked;
    }
}

impl Pretty for GitRepo {
    fn pretty(&self, mode: IconMode) -> Option<String> {
        let mut res = vec![];

        if let Some(state) = &self.state {
            res.push(format!("{}|", state.pretty(mode).unwrap_or_default()));
        }

        let head = self.head.pretty(mode).unwrap_or_default();
        res.push(head);

        if let HeadKind::Branch(local) = &self.head.kind
            && let Some(Remote { branch: remote, .. }) = &self.remote
            && local != remote
        {
            res.push(format!(":{remote}"));
        }

        if let Some(Remote { exists: false, .. }) = &self.remote {
            res.push("?".into());
        }

        for (icon, val) in [
            (GitIcon::Stashes, self.stashes),
            (GitIcon::Behind, self.behind),
            (GitIcon::Ahead, self.ahead),
        ] {
            if val != 0 {
                res.push(format!(" {}{}", icon.icon(mode), val));
            }
        }

        let text = "[".to_owned() + &res.join("") + "]";
        Some(
            text.visible()
                .colorize_with(self.head.ref_name().as_ref()) //.pink()
                .bold()
                .with_reset()
                .invisible()
                .to_string(),
        )
    }
}

impl Pretty for GitTree {
    fn pretty(&self, mode: IconMode) -> Option<String> {
        let vec = [
            (GitIcon::Conflict, self.unmerged),
            (GitIcon::Staged, self.staged),
            (GitIcon::Dirty, self.dirty),
            (GitIcon::Untracked, self.untracked),
        ]
        .into_iter()
        .filter(|(_, val)| *val != 0)
        .map(|(s, val)| format!("{}{}", s.icon(mode), val))
        .collect::<Vec<_>>();

        if vec.is_empty() {
            None
        } else {
            let text = "[".to_owned() + &vec.join(" ") + "]";
            Some(text.visible().pink().with_reset().invisible().to_string())
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
    fn icon(&self, mode: IconMode) -> &'static str {
        use IconMode::*;
        match &self {
            Self::Ahead => match mode {
                Text => "^",
                Icons | MinimalIcons => " ",
            },
            Self::Behind => match mode {
                Text => "v",
                Icons | MinimalIcons => " ",
            },
            Self::Stashes => match mode {
                Text => "*",
                Icons | MinimalIcons => " ",
            },
            Self::Conflict => match mode {
                Text => "=",
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
