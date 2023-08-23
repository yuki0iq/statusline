use crate::file;
use crate::prompt::Prompt;
use anyhow::{anyhow, bail, Result};
use mmarinus::{perms, Map, Private};
use std::{
    ffi::OsStr,
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

fn parse_ref_by_name<T: AsRef<str>>(name: T) -> Head {
    if let Some(name) = name.as_ref().trim().strip_prefix("refs/heads/") {
        Head::Branch(name.to_owned())
    } else {
        Head::Unknown
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

enum Head {
    Branch(String),
    Commit(String),
    Unknown,
}

impl Head {
    fn pretty(&self, root: &Path, prompt: &Prompt) -> String {
        match &self {
            Head::Branch(name) => format!("{} {}", prompt.on_branch(), name),
            Head::Commit(id) => {
                format!("{} {}", prompt.at_commit(), &id[..abbrev_commit(root, id)])
                // TODO show tag?
            }
            _ => "<unknown>".to_string(),
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

    fn pretty(&self, prompt: &Prompt) -> String {
        match self {
            State::Bisecting => prompt.git_bisect().to_string(),
            State::Reverting { head } => format!("{} {}", prompt.git_revert(), head),
            State::CherryPicking { head } => format!("{} {}", prompt.git_cherry(), head),
            State::Merging { head } => format!("{} {}", prompt.git_merge(), head),
            State::Rebasing { done, todo } => {
                format!("{} {}/{}", prompt.git_rebase(), done, done + todo)
            }
        }
    }
}

fn get_remote(root: &Path, head: &Head) -> Option<(String, String)> {
    let Head::Branch(br) = head else {
        return None;
    };

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
    /// Working tree path
    pub tree: PathBuf,
    root: PathBuf,
    head: Head,
    remote: Option<(String, String)>,
    stashes: usize,
    state: Option<State>,
}

/// Additional git status information, about branch tracking and working tree state
pub struct GitStatusExtended {
    behind: usize,
    ahead: usize,
    unmerged: usize,
    staged: usize,
    dirty: usize,
    untracked: usize,
}

impl GitStatus {
    /// Get git status for current working directory --- for the innermost repository or submodule
    pub fn build(workdir: &Path) -> Result<GitStatus> {
        let dotgit = file::upfind(workdir, ".git")?;
        let tree = dotgit.parent().unwrap().to_path_buf();
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

        // eprintln!("ok tree {tree:?} | {root:?}");

        let head_path = root.join("HEAD");

        let head = if head_path.is_symlink() {
            parse_ref_by_name(
                fs::read_link(head_path)?
                    .to_str()
                    .ok_or(Error::from(ErrorKind::InvalidFilename))?,
            )
        } else {
            let head = fs::read_to_string(root.join("HEAD"))?;
            if let Some(rest) = head.strip_prefix("ref:") {
                parse_ref_by_name(rest)
            } else {
                Head::Commit(
                    head.split_whitespace()
                        .next()
                        .unwrap_or_default()
                        .to_owned(),
                )
            }
        };

        let remote = get_remote(&root, &head);

        let stash_path = root.join("logs/refs/stash");
        // eprintln!("try find stashes in {stash_path:?}");
        let stashes = fs::File::open(stash_path)
            .map(|file| BufReader::new(file).lines().count())
            .unwrap_or(0);

        let state = State::from_env(&root);

        Ok(GitStatus {
            tree,
            root,
            head,
            remote,
            stashes,
            state,
        })
    }

    /// Get extended git informtion, if possible. Relies on `git` executable
    pub fn extended(&self) -> Option<GitStatusExtended> {
        let out = Command::new("git")
            .arg("-C")
            .arg(&self.tree)
            .arg("status")
            .arg("--porcelain=2")
            .output()
            .ok()?;
        let lines = out.stdout.split(|&c| c == b'\n').peekable();

        let (ahead, behind) = self.get_ahead_behind().unwrap_or((0, 0));

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

        Some(GitStatusExtended {
            behind,
            ahead,
            unmerged,
            staged,
            dirty,
            untracked,
        })
    }

    fn get_ahead_behind(&self) -> Result<(usize, usize)> {
        let (Head::Branch(head), Some((name, branch))) = (&self.head, &self.remote) else {
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

    /// Pretty-formats git status with respect to the chosen mode
    pub fn pretty(&self, prompt: &Prompt) -> String {
        let mut res = vec![];

        if let Some(state) = &self.state {
            res.push(format!("{}|", state.pretty(prompt)));
        }

        let head = self.head.pretty(&self.root, prompt);
        res.push(head);

        match (&self.head, &self.remote) {
            (Head::Branch(head), Some((_, remote))) if head != remote => {
                res.push(format!(":{}", remote));
            }
            _ => (),
        };

        for (s, val) in [("*", self.stashes)] {
            if val != 0 {
                res.push(format!(" {}{}", s, val));
            }
        }

        res.join("")
    }
}

impl GitStatusExtended {
    /// Pretty-formats extended git status with respect to the chosen mode
    pub fn pretty(&self, prompt: &Prompt) -> String {
        [
            (prompt.behind(), self.behind),
            (prompt.ahead(), self.ahead),
            (prompt.conflict(), self.unmerged),
            (prompt.staged(), self.staged),
            (prompt.dirty(), self.dirty),
            (prompt.untracked(), self.untracked),
        ]
        .into_iter()
        .filter(|(_, val)| val != &0)
        .map(|(s, val)| format!(" {}{}", s, val))
        .collect::<Vec<_>>()
        .join("")
    }
}
