use crate::file::upfind;
use std::{
    fmt,
    path::{Path, PathBuf},
    process::Command,
};

pub fn git_info(workdir: &Path) -> Option<(PathBuf, GitStatus)> {
    Some((
        upfind(workdir, ".git")?.parent()?.to_path_buf(),
        GitStatus::build()?,
    ))
}

/*
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
    6 unstaged   -> .X
    7 untracked   -> ?
*/
pub struct GitStatus {
    branch: String,
    remote_branch: Option<String>,
    behind: u32,
    ahead: u32,
    stashes: u32,
    unmerged: u32,
    staged: u32,
    unstaged: u32,
    untracked: u32,
}

impl GitStatus {
    pub fn build() -> Option<GitStatus> {
        let out_bytes = Command::new("git")
            .args(["status", "--porcelain=2", "--branch", "--show-stash"])
            .output()
            .ok()?
            .stdout;
        let out = String::from_utf8(out_bytes).ok()?;
        let mut lines = out.lines().peekable();

        let mut branch = String::new();
        let mut remote_branch: Option<String> = None;
        let mut behind = 0;
        let mut ahead = 0;
        let mut stashes = 0;

        while let Some(cmd) = lines.peek().and_then(|x| x.strip_prefix("# ")) {
            lines.next();
            if let Some(stash) = cmd.strip_prefix("stash ") {
                stashes = stash.parse().unwrap();
            } else if let Some(branches) = cmd.strip_prefix("branch.") {
                let mut words = branches.split(' ');
                match words.next().unwrap() {
                    "head" => {
                        branch = words.next().unwrap().to_owned();
                    }
                    "upstream" => {
                        let remote: Vec<_> = words.next().unwrap().split('/').collect();
                        let (_upstream, branch) = (remote[0], remote[1]);
                        remote_branch = Some(branch.to_owned());
                    }
                    "ab" => {
                        let diff: Vec<_> = words.map(|word| word[1..].parse().unwrap()).collect();
                        (ahead, behind) = (diff[0], diff[1]);
                    }
                    _ => (),
                }
            }
        }

        let mut unmerged = 0;
        let mut staged = 0;
        let mut unstaged = 0;
        let mut untracked = 0;

        for line in lines {
            let words: Vec<_> = line.split(' ').take(2).collect();
            let (id, pat) = (words[0], words[1]);
            match (id, pat) {
                ("?", _) => {
                    untracked += 1;
                }
                ("u", _) => {
                    unmerged += 1;
                }
                (_, pat) => {
                    if "M. T. A. D. R. C. U.".contains(pat) {
                        staged += 1;
                    } else {
                        unstaged += 1;
                    }
                }
            }
        }

        Some(GitStatus {
            branch,
            remote_branch,
            behind,
            ahead,
            stashes,
            unmerged,
            staged,
            unstaged,
            untracked,
        })
    }
}

impl fmt::Display for GitStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut entries = vec![];

        entries.push(if let Some(remote) = &self.remote_branch && !self.branch.eq(remote) {
            format!("{}:{}", &self.branch, &remote)
        } else {
            self.branch.clone()
        });

        let mut add_if = |s: &str, val: u32| {
            if 0 != val {
                entries.push(format!("{}{}", s, val));
            }
        };

        add_if("v", self.behind);
        add_if("^", self.ahead);
        add_if("*", self.stashes);
        add_if("~", self.unmerged);
        add_if("+", self.staged);
        add_if("!", self.unstaged);
        add_if("?", self.untracked);

        write!(f, "{}", entries.join(" "))
    }
}
