use crate::file;
use crate::prompt::Prompt;
use anyhow::Result;
use hex::FromHex;
use mmarinus::{perms, Map, Private};
use std::{
    fs,
    io::{BufRead, BufReader, Error, ErrorKind},
    iter,
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

fn load_objects(root: &Path, prefix: &str) -> Result<Vec<String>> {
    Ok(fs::read_dir(root.join("objects").join(prefix))?
        .map(|res| res.map(|e| String::from(e.file_name().to_string_lossy())))
        .collect::<Result<Vec<_>, _>>()?)
}

fn objects_dir_len(root: &Path, prefix: &str, rest: &str) -> Result<usize> {
    // Find len from ".git/objects/xx/..."
    Ok(2 + load_objects(root, prefix)?
        .iter()
        .map(|val| 1 + lcp(val.as_str(), rest))
        .max()
        .unwrap_or(0))
}

fn packed_objects_len(root: &Path, commit: &str) -> Result<usize> {
    let commit = <[u8; 20]>::from_hex(commit)?;

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

        // Should contain 0x102 ints (magic, version and fanout)
        if map.size() < 0x408 {
            continue;
        }

        let read_int_at_index =
            |pos: usize| u32::from_be_bytes(*map[pos * 4..].first_chunk::<4>().unwrap());

        // Magic int is 0xFF744F63 ('\377tOc')
        // probably should be read as "table of contents" which this index is
        if read_int_at_index(0) != 0xFF744F63 {
            continue;
        }

        // Only version 2 is supported
        if read_int_at_index(1) != 0x00000002 {
            continue;
        }

        // eprintln!("magic + version ok");
        // [0x0008 -- 0x0408] is fanout table as [u32, 256]
        // where `table[i]` is count of objects with `prefix <= i`
        // object range is from `table[i-1]` to `table[i] - 1` including both borders
        let prefix = commit[0] as usize;
        let begin = if prefix == 0 {
            0
        } else {
            read_int_at_index(prefix + 1)
        } as usize;
        let end = read_int_at_index(prefix + 2) as usize;

        // begin and end are sha1 *indexes* and not positions of their beginning
        if begin == end {
            continue;
        }

        // check that right is fully readable
        if map.size() < 0x408 + 20 * end {
            continue;
        }

        let hashes: &[[u8; 20]] = unsafe { std::mem::transmute(&map[0x408..]) };

        //eprintln!("left and right are {left:?} and {right:?}");

        let index = hashes[begin..end].partition_point(|hash| hash < &commit);
        if index > 0 {
            res = res.max(1 + lcp_bytes(&hashes[begin + index - 1], &commit));
        }
        if index < end - begin {
            res = res.max(
                1 + lcp_bytes(
                    &hashes[begin + index + (hashes[begin + index] == commit) as usize],
                    &commit,
                ),
            );
        }
    }
    //eprintln!("packed: {res:?}");
    //eprintln!("");
    Ok(res)
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
                let (prefix, rest) = id.split_at(2);

                let mut abbrev_len = 4;
                if let Ok(x) = objects_dir_len(&root, &prefix, &rest) {
                    abbrev_len = abbrev_len.max(x);
                }
                if let Ok(x) = packed_objects_len(&root, &id) {
                    abbrev_len = abbrev_len.max(x);
                }

                format!(
                    "{} {}",
                    prompt.at_commit(),
                    id.split_at(abbrev_len).0
                ) // TODO show tag?
            }
            _ => "<unknown>".to_string(),
        }
    }
}

pub struct GitStatus {
    pub tree: PathBuf,
    root: PathBuf,
    head: Head,
    remote_branch: Option<String>,
    stashes: usize,
}

pub struct GitStatusExtended {
    behind: u32,
    ahead: u32,
    unmerged: u32,
    staged: u32,
    dirty: u32,
    untracked: u32,
}

impl GitStatus {
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

        let remote_branch = if let Head::Branch(br) = &head {
            let section = format!("[branch \"{br}\"]");
            // eprintln!("section: {section} | {:?}", root.join("config"));
            BufReader::new(fs::File::open(root.join("config"))?)
                .lines()
                .skip_while(|x| match x {
                    Ok(x) => x != &section,
                    _ => false,
                })
                .skip(1)
                .take_while(|x| match x {
                    Ok(x) if x.starts_with("\t") => true,
                    _ => false,
                })
                .find_map(|x| match x {
                    Ok(x) => x
                        .strip_prefix("\tmerge = refs/heads/")
                        .map(|x| x.to_string()),
                    _ => None,
                })
        } else {
            None
        };

        let stash_path = root.join("logs/refs/stash");
        // eprintln!("try find stashes in {stash_path:?}");
        let stashes = fs::File::open(stash_path)
            .map(|file| BufReader::new(file).lines().count())
            .unwrap_or(0);

        Ok(GitStatus {
            tree,
            root,
            head,
            remote_branch,
            stashes,
        })
    }

    pub fn extended(&self) -> Option<GitStatusExtended> {
        let out = Command::new("git")
            .args([
                "-C",
                self.tree.to_str()?,
                "status",
                "--porcelain=2",
                "--branch",
            ])
            .output()
            .ok()?;
        let mut lines = out.stdout.split(|&c| c == b'\n').peekable();

        let mut behind: u32 = 0;
        let mut ahead: u32 = 0;

        while let Some(cmd) = lines.peek().and_then(|x| x.strip_prefix(b"# ")) {
            lines.next();
            if let Some(branches) = cmd.strip_prefix(b"branch.ab ") {
                let diff = branches
                    .split(|&c| c == b' ')
                    .map(|word| std::str::from_utf8(&word[1..]).ok()?.parse().ok())
                    .collect::<Option<Vec<_>>>()?;
                if diff.len() != 2 {
                    return None;
                }
                (ahead, behind) = (diff[0], diff[1]);
            }
        }

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

    pub fn pretty(&self, prompt: &Prompt) -> String {
        let head = self.head.pretty(&self.root, &prompt);
        let mut res = vec![head];

        match (&self.head, &self.remote_branch) {
            (Head::Branch(head), Some(remote)) if head.ne(remote) => {
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
