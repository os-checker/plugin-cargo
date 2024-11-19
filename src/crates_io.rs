use crate::prelude::*;

const REPO: &str = "https://github.com/rust-lang/crates.io-index.git";
const DIR: &str = "crates.io-index";

#[test]
fn test_crates_io_index() -> Result<()> {
    let mut dir = crate::repo::local_base_dir().to_owned();
    duct::cmd!("git", "clone", REPO, DIR, "--depth", "1")
        .dir(&dir)
        .run()?;

    dir.push(DIR);
    duct::cmd!("ls", "-alh", dir).run()?;

    Ok(())
}
