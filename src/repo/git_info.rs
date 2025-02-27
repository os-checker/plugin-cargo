use plugin::prelude::{cmd, duct, Result, Timestamp, Utf8Path};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GitInfo {
    pub last_commit: Timestamp,
    pub sha: String,
    pub branch: String,
}

impl GitInfo {
    pub fn new(root: &Utf8Path) -> Result<Self> {
        let last_commit = last_commit_time(root)?;
        let sha = head_sha(root)?;
        let branch = current_branch(root)?;
        Ok(Self {
            last_commit,
            sha,
            branch,
        })
    }
}

fn run(expr: duct::Expression, root: &Utf8Path) -> Result<String> {
    Ok(expr.dir(root).read()?.trim().to_owned())
}

fn last_commit_time(root: &Utf8Path) -> Result<Timestamp> {
    let cmd = cmd!("git", "log", "-1", "--pretty=format:%ct");
    // git log -1 --format="%ct"
    let unix_timestamp = run(cmd, root)?.parse()?;
    Ok(Timestamp::from_second(unix_timestamp)?)
}

fn head_sha(root: &Utf8Path) -> Result<String> {
    // git rev-parse HEAD
    run(cmd!("git", "rev-parse", "HEAD"), root)
}

fn current_branch(root: &Utf8Path) -> Result<String> {
    // git branch --show-current
    run(cmd!("git", "branch", "--show-current"), root)
}

#[test]
fn git_info() -> Result<()> {
    let root = ".".into();
    dbg!(GitInfo::new(root)?);
    Ok(())
}
