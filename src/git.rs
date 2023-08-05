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
    let len = a.len().min(b.len());
    for i in 0..len {
        if (a[i] >> 4) != (b[i] >> 4) {
            return 1 + 2 * i;
        }
        if (a[i] & 15) != (b[i] & 15) {
            return 2 + 2 * i;
        }
    }
    0
}

fn load_objects(root: &Path, prefix: &str) -> Result<Vec<String>> {
    Ok(fs::read_dir(root.join(format!("objects/{prefix}")))?
        .map(|res| res.map(|e| String::from(e.file_name().to_string_lossy())))
        .collect::<Result<Vec<_>, _>>()?)
}

fn objects_dir_len(root: &Path, prefix: &str, rest: &str) -> Result<usize> {
    // Find len from ".git/objects/xx/..."

    let objects = load_objects(&root, &prefix)?;
    let mut ans = 0;

    let lesser = objects.iter().filter(|obj| &obj[..] < rest).max();
    if let Some(val) = lesser {
        ans = ans.max(1 + lcp(&val[..], &rest));
    }

    let greater = objects.iter().filter(|obj| &obj[..] > rest).min();
    if let Some(val) = greater {
        ans = ans.max(1 + lcp(&val[..], &rest));
    }

    // eprintln!("objdir: {ans:?}");
    Ok(ans)
}

fn packed_objects_len(root: &Path, prefix: &str, commit: &str) -> Result<usize> {
    // TODO packed objects
    let mut res = 0;
    for entry in fs::read_dir(root.join("objects/pack"))? {
        let path = entry?.path();
        // eprintln!("entry {path:?}");
        if let Some(ext) = path.extension() && ext == "idx" {
            let map = Map::load(path, Private, perms::Read)?;
            // eprintln!("mmaped");

            // Git packed objects index file format is easy
            // See https://github.com/purplesyringa/gitcenter -> main/dist/js/git.md

            // Should contain 0x102 ints (magic, version and fanout)
            if map.size() < 0x408 { continue; }

            let read_int_at = |pos: usize| {
                let left = pos * 4;
                let right = left + 4;
                let value = map[left..right].first_chunk::<4>().unwrap();
                u32::from_be_bytes(*value)
            };

            // Magic int is 0xFF744F63 ('\377tOc')
            // probably should be read as "table of contents" which this index is
            if read_int_at(0) != 0xFF744F63 {
                continue;
            }

            // Only version 2 is supported
            if read_int_at(1) != 0x00000002 {
                continue;
            }

            // eprintln!("magic + version ok"); 
            // [0x0008 -- 0x0408] is fanout table as [u32, 256]
            // where `table[i]` is count of objects with `prefix <= i`
            // object range is from `table[i-1]` to `table[i] - 1` including both borders
            let prefix = usize::from_str_radix(&prefix[..], 16)?;
            let left = if prefix == 0 { 0 } else { read_int_at(prefix + 1) } as usize;
            let right = read_int_at(prefix + 2) as usize;

            // left and right are sha1 *indexes* and not positions of their beginning
            if left == right { continue; }
            let right = right - 1 as usize;

            // check that right is fully readable
            if map.size() < 0x408 + 20*(right + 1) { continue; }

            let hash_pos = |pos: usize| 0x408 + pos * 20;
            let hash = |pos: usize| {
                let pos = hash_pos(pos);
                map[pos..pos+20].first_chunk::<20>().unwrap()
            };
            let commit = <[u8; 20]>::from_hex(commit)?;
            //eprintln!("left and right are {left:?} and {right:?}");

            let objects = (left..=right).map(|i| hash(i)); // TODO binary search

            let lesser = objects.clone().filter(|&&obj| obj < commit).max();
            //eprintln!("lesser: {lesser:?}");
            if let Some(val) = lesser {
                res = res.max(lcp_bytes(&val[1..], &commit[1..]));
            }

            let greater = objects.filter(|&&obj| obj > commit).min();
            //eprintln!("greater: {greater:?}");
            if let Some(val) = greater {
                res = res.max(lcp_bytes(&val[1..], &commit[1..]));
            }
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

                let mut abbrev_len = 2;
                if let Ok(x) = objects_dir_len(&root, &prefix, &rest) {
                    abbrev_len = abbrev_len.max(x);
                }
                if let Ok(x) = packed_objects_len(&root, &prefix, &id) {
                    abbrev_len = abbrev_len.max(x);
                }

                format!(
                    "{} {}{}",
                    prompt.at_commit(),
                    &prefix,
                    rest.split_at(abbrev_len).0
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
