use crate::file::upfind;
use std::{
    fmt, fs,
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

enum Head {
    Branch(String),
    Commit(String),
    Unknown,
}

impl fmt::Display for Head {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Head::Branch(name) => write!(f, "{}", name),
            Head::Commit(id) => write!(f, "{}", &id[..5]), // TODO
            _ => Ok(()),
        }
    }
}

pub struct GitStatus {
    pub tree: PathBuf,
    head: Head,
    remote_branch: Option<String>,
    stashes: u32,
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
            PathBuf::from(
                fs::read_to_string(&tree)
                    .ok()?
                    .strip_prefix("gitdir: ")?
                    .trim_end_matches(&['\r', '\n']),
            )
        } else {
            dotgit
        };

        // println!("ok {tree:?} | {root:?}");

        let head = fs::read_to_string(root.join("HEAD")).ok()?;
        // println!("head is {head:?}");
        let head = if let Some(rest) = head.strip_prefix("ref:") {
            if let Some(name) = rest.trim().strip_prefix("refs/heads/") {
                Head::Branch(name.to_owned())
            } else {
                Head::Unknown
            }
        } else {
            Head::Commit(head.split_whitespace().next()?.to_owned())
        };

        let remote_branch = None;
        let stashes = 0;

        Some(GitStatus {
            tree,
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
}

impl fmt::Display for GitStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let head = self.head.to_string();
        write!(f, "{}", head)?;

        if let Some(remote) = &self.remote_branch && head != *remote {
            write!(f, ":{}", remote)?;
        }

        for (s, val) in [("*", self.stashes)] {
            if val != 0 {
                write!(f, " {}{}", s, val)?;
            }
        }

        Ok(())
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
