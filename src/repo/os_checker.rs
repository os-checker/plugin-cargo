use crate::prelude::*;
use duct::cmd;
use os_checker_types::layout::ListTargets;

pub fn run(repo: &str) -> Result<Vec<ListTargets>> {
    // multiple config paths are separated by ` `
    let configs =
        std::env::var("CONFIGS").with_context(|| "Must specify `CONFIGS` environment variable.")?;

    let dir = super::git_clone_dir();
    let mut args = vec!["layout", "--base-dir", dir.as_str(), "--list-targets", repo];

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

    Ok(serde_json::from_reader(output.stdout.as_slice())?)
}

#[test]
fn test_sel4() -> Result<()> {
    let repo = "seL4/rust-sel4";
    dbg!(run(repo)?);
    Ok(())
}
