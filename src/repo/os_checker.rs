use crate::prelude::*;
use duct::cmd;
use os_checker_types::layout::ListTargets;

pub type PkgTargets = IndexMap<XString, Vec<String>>;

pub fn run(user_repo: &str) -> Result<PkgTargets> {
    // multiple config paths are separated by ` `
    let configs =
        std::env::var("CONFIGS").with_context(|| "Must specify `CONFIGS` environment variable.")?;

    let dir = super::git_clone_dir();
    let mut args = vec![
        "layout",
        "--base-dir",
        dir.as_str(),
        "--list-targets",
        user_repo,
    ];

    for config in configs.split(" ").map(|s| s.trim()) {
        if !config.is_empty() {
            args.extend(["--config", config]);
        }
    }

    let output = cmd("os-checker", args)
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
    // TODO: remove dir
    Ok(())
}
