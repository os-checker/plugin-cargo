use super::local_base_dir;
use crate::prelude::*;
use duct::cmd;
use os_checker_types::layout::ListTargets;

pub type PkgTargets = IndexMap<XString, Vec<String>>;

pub fn run(user_repo: &str) -> Result<PkgTargets> {
    let dir = local_base_dir();

    // OS_CHECKER_CONFIGS is inherented
    let output = cmd!(
        "os-checker",
        "layout",
        "--base-dir",
        dir,
        "--list-targets",
        user_repo,
    )
    .env_remove("RUST_LOG")
    .stdout_capture()
    .stderr_capture()
    .unchecked()
    .run()?;

    ensure!(
        output.status.success(),
        "{}",
        std::str::from_utf8(&output.stderr)?
    );

    let v: Vec<ListTargets> = serde_json::from_reader(output.stdout.as_slice())?;
    Ok(list_to_map(v))
}

/// returns `Map<PkgName, TargetTriples>`
fn list_to_map(v: Vec<ListTargets>) -> PkgTargets {
    v.into_iter().map(|l| (l.pkg, l.targets)).collect()
}

#[test]
fn test_sel4() -> Result<()> {
    let repo = "seL4/rust-sel4";
    dbg!(run(repo)?);
    std::fs::remove_dir_all(local_base_dir())?;
    Ok(())
}
