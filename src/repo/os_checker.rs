use super::local_base_dir;
use os_checker_types::layout::ListTargets;
use plugin::prelude::*;

pub type PkgTargets = IndexMap<XString, Vec<String>>;

pub fn run(user_repo: &str) -> Result<PkgTargets> {
    const OUT: &str = "layout.json";
    let dir = local_base_dir();

    // OS_CHECKER_CONFIGS is inherented
    let output = cmd!(
        "os-checker",
        "layout",
        "--base-dir",
        dir,
        "--list-targets",
        user_repo,
        "--out",
        OUT
    )
    .env_remove("RUST_LOG")
    .stderr_capture()
    .unchecked()
    .run()?;

    ensure!(
        output.status.success(),
        "{}",
        std::str::from_utf8(&output.stderr)?
    );

    let targets = std::fs::read_to_string(OUT)
        .with_context(|| format!("Layout output file {OUT} doesn't exist."))?;
    println!("targets=\n{targets}");
    let v: Vec<ListTargets> = serde_json::from_str(&targets)?;
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
