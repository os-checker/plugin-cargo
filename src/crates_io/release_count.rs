use crate::prelude::*;
use cargo_metadata::semver::Version;
use serde::Deserialize;

fn url(pkg: &str) -> String {
    const PREFIX: &str = "https://index.crates.io";

    // ref: https://doc.rust-lang.org/cargo/reference/registry-index.html#index-files
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

    // e.g. https://raw.githubusercontent.com/rust-lang/crates.io-index/refs/heads/master/os/-c/os-checker
    let mut buf = String::with_capacity(128);
    buf.push_str(PREFIX);

    for c in components {
        buf.push('/');
        buf.push_str(c);
    }

    buf
}

#[derive(Debug, Deserialize)]
pub struct Data {
    pub vers: Version,
}

fn parse_data(index_file: &str) -> Result<Vec<Data>> {
    serde_json::Deserializer::from_str(index_file)
        .into_iter()
        .map(|val| Ok(val?))
        .collect()
}

/// NOTE: the error may be due to network failure or invalid text
pub fn get_data(pkg: &str) -> Result<Vec<Data>> {
    let url = url(pkg);
    info!("wget {url}");

    let output = duct::cmd!("wget", &url, "-O", "-")
        .stdout_capture()
        .stderr_null()
        .run()?;

    let text = std::str::from_utf8(&output.stdout)?.trim();
    parse_data(text)
}

/// None means no release found; 0 is an invalid value because there at least one
/// release if found.
pub fn get_release_count(pkg: &str) -> Option<usize> {
    let count = get_data(pkg).ok()?.len();
    if count == 0 {
        error!(pkg, "count is an invalid value 0");
    }
    Some(count)
}

#[test]
fn test_get_release_count() {
    dbg!(get_release_count("os-checker"));
}
