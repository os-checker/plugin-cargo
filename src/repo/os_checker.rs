use crate::prelude::*;
use duct::cmd;
use os_checker_types::layout::ListTargets;

pub fn run(repo: &str) -> Result<Vec<ListTargets>> {
    let dir = super::git_clone_dir();
    let output = cmd!(
        "os-checker",
        "layout",
        "--base-dir",
        dir,
        "--list-targets",
        repo
    )
    .stdout_capture()
    .stderr_capture()
    .unchecked()
    .run()?;

    ensure!(
        output.status.success(),
        "{}",
        std::str::from_utf8(&output.stderr)?
    );

    Ok(serde_json::from_reader(output.stdout.as_slice())?)
}

#[test]
fn test_sel4() -> Result<()> {
    let repo = "seL4/rust-sel4";
    dbg!(run(repo)?);
    Ok(())
}
