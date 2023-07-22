use crate::file::upfind;
use crate::prompt::Prompt;
use std::{
    cmp, fmt, fs,
    io::{BufRead, BufReader},
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

fn parse_ref_by_name(name: &str) -> Head {
    if let Some(name) = name.trim().strip_prefix("refs/heads/") {
        Head::Branch(name.to_owned())
    } else {
        Head::Unknown
    }
}

fn lcp<T: AsRef<str>>(a: T, b: T) -> usize {
    iter::zip(a.as_ref().chars(), b.as_ref().chars())
        .position(|(a, b)| a != b)
        .unwrap_or(0)
}

fn load_objects(root: &Path, prefix: &str) -> Option<Vec<String>> {
    Some(
        fs::read_dir(root.join(format!("objects/{prefix}")))
            .ok()?
            .map(|res| res.map(|e| String::from(e.path().to_string_lossy())))
            .collect::<Result<Vec<_>, _>>()
            .ok()?,
    )
}

fn objects_dir_len<T: AsRef<str> + cmp::Ord>(objects: &[T], rest: &str) -> Option<usize> {
    // Find len from ".git/objects/xx/..."
    let len = objects.len();
    if len == 1 {
        Some(0)
    } else if len == 2 {
        Some(1 + lcp(&objects[0], &objects[1]))
    } else {
        let idx = objects.binary_search_by(|x| x.as_ref().cmp(rest)).ok()?;
        let left = if idx != 0 { idx - 1 } else { idx };
        let right = if idx != objects.len() { idx + 1 } else { idx };
        Some(1 + lcp(&objects[left], &objects[right]))
    }
}

enum Head {
    Branch(String),
    Commit(String),
    Unknown,
}

impl Head {
    fn pretty<T: AsRef<str> + cmp::Ord>(&self, objects: &[T], prompt: &Prompt) -> String {
        match &self {
            Head::Branch(name) => format!("{} {}", prompt.on_branch(), name),
            Head::Commit(id) => {
                let (prefix, rest) = id.split_at(2);

                let abbrev_len = [Some(2), objects_dir_len(&objects, &rest)]
                    .iter()
                    .filter_map(|&x| x)
                    .reduce(cmp::max)
                    .unwrap();

                format!(
                    "{} {}{}",
                    prompt.at_commit(),
                    &prefix,
                    rest.split_at(abbrev_len).0
                ) // TODO object index? show tag?
            }
            _ => "<unknown>".to_string(),
        }
    }
}

pub struct GitStatus {
    pub tree: PathBuf,
    // root: PathBuf,
    head: Head,
    remote_branch: Option<String>,
    stashes: usize,
    objects: Vec<String>,
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
    pub fn build(workdir: &Path) -> Option<GitStatus> {
        let dotgit = upfind(workdir, ".git")?;
        let tree = dotgit.parent()?.to_path_buf();
        let root = if dotgit.is_file() {
            tree.join(
                fs::read_to_string(&dotgit)
                    .ok()?
                    .strip_prefix("gitdir: ")?
                    .trim_end_matches(&['\r', '\n']),
            )
        } else {
            dotgit
        };

        // eprintln!("ok tree {tree:?} | {root:?}");

        let head_path = root.join("HEAD");

        let head = if head_path.is_symlink() {
            parse_ref_by_name(fs::read_link(head_path).ok()?.to_str()?)
        } else {
            let head = fs::read_to_string(root.join("HEAD")).ok()?;
            if let Some(rest) = head.strip_prefix("ref:") {
                parse_ref_by_name(rest)
            } else {
                Head::Commit(head.split_whitespace().next()?.to_owned())
            }
        };

        let remote_branch = if let Head::Branch(br) = &head {
            let section = format!("[branch \"{br}\"]");
            // eprintln!("section: {section} | {:?}", root.join("config"));
            BufReader::new(fs::File::open(root.join("config")).ok()?)
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
            .ok()
            .map(|file| BufReader::new(file).lines().count())
            .unwrap_or(0);

        let objects = if let Head::Commit(id) = &head {
            let mut obj = load_objects(&root, &id[..2]).unwrap_or_default();
            obj.sort();
            obj
        } else {
            vec![]
        };

        Some(GitStatus {
            tree,
            // root,
            head,
            remote_branch,
            stashes,
            objects,
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

        println!("ahead and behind is {ahead} {behind}");

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
        let head = self.head.pretty(&self.objects, &prompt);
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

impl fmt::Display for GitStatusExtended {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (s, val) in [
            ("v", self.behind),
            ("^", self.ahead),
            ("~", self.unmerged),
            ("+", self.staged),
            ("!", self.dirty),
            ("?", self.untracked),
        ] {
            if val != 0 {
                write!(f, " {}{}", s, val)?;
            }
        }

        Ok(())
    }
}
