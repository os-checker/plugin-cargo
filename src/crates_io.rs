use crate::prelude::*;
use std::sync::LazyLock;

const REPO: &str = "https://github.com/rust-lang/crates.io-index.git";

/// Returns the pkg file path if exists
// ref: https://doc.rust-lang.org/cargo/reference/registry-index.html#index-files
fn search_pkg_file(pkg: &str, dir: &Utf8Path) -> Option<Utf8PathBuf> {
    let components = match pkg.len() {
        1 => &["1", pkg][..],
        2 => &["2", pkg],
        3 => {
            let (a, b) = pkg.split_at(1);
            &["3", a, b, pkg]
        }
        _ => {
            let (a, b) = pkg.split_at(2);
            let (b, _) = b.split_at(2);
            &[a, b, pkg]
        }
    };
    let path = {
        let mut path = dir.to_owned();
        path.extend(components);
        path
    };
    path.exists().then_some(path)
}

static DIR: LazyLock<Utf8PathBuf> = LazyLock::new(|| {
    let dir = {
        const DIR: &str = "crates.io-index";
        let mut dir = crate::repo::local_base_dir().to_owned();
        dir.push(DIR);
        dir
    };
    duct::cmd!("git", "clone", REPO, &dir, "--depth", "1")
        .run()
        .unwrap();
    dir
});

#[test]
fn test_search_pkg_file() -> Result<()> {
    dbg!(search_pkg_file("os-checker", &DIR).unwrap());

    Ok(())
}
