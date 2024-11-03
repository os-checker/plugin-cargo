pub mod prelude {
    pub use camino::{Utf8Path, Utf8PathBuf};
    pub use cargo_metadata::Metadata;
    pub use eyre::{Context, Result};
    pub use indexmap::IndexMap;
}

#[macro_use]
extern crate eyre;
#[macro_use]
extern crate tracing;

pub mod logger;
pub mod repo;

pub const BASE: &str = "tmp";

use prelude::*;
use std::fs;

pub fn write_json<T: serde::Serialize>(path: &Utf8Path, val: &T) -> Result<()> {
    let _span = error_span!("write_json", ?path).entered();
    fs::create_dir_all(path.parent().unwrap())?;
    serde_json::to_writer_pretty(fs::File::create(path)?, val)?;
    Ok(())
}
