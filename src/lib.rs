#[macro_use]
extern crate eyre;
#[macro_use]
extern crate tracing;

use plugin::prelude::*;

pub mod crates_io;
pub mod database;
pub mod nextest;
pub mod repo;

pub const BASE: &str = "tmp";

use std::fs;

pub fn write_json<T: serde::Serialize>(path: &Utf8Path, val: &T) -> Result<()> {
    let _span = error_span!("write_json", ?path).entered();
    fs::create_dir_all(path.parent().unwrap())?;
    serde_json::to_writer_pretty(fs::File::create(path)?, val)?;
    Ok(())
}

/// Get the list of repos, searching in the following order:
/// * the first argument: a json path to a vec of repo string
/// * or read from the result of `os-checker config --list-repos`
pub fn repos() -> Result<Vec<String>> {
    let arg = std::env::args().nth(1);

    let text = match arg.as_deref() {
        Some(list_json) => {
            let path = Utf8Path::new(list_json);
            fs::read_to_string(path)?
        }
        None => duct::cmd!("os-checker", "config", "--list-repos")
            .env_remove("RUST_LOG")
            .read()?,
    };

    info!(text);
    Ok(serde_json::from_str(&text)?)
}
