mod prelude {
    pub use camino::{Utf8Path, Utf8PathBuf};
    pub use cargo_metadata::Metadata;
    pub use eyre::{Context, Result};
    pub use indexmap::IndexMap;
}

use eyre::ContextCompat;
use prelude::*;

#[macro_use]
extern crate eyre;

mod logger;
mod repo;

const BASE: &str = "tmp";

fn main() -> Result<()> {
    logger::init();

    let arg = std::env::args()
        .nth(1)
        .with_context(|| "the first argument should be a json path")?;
    let list_json = Utf8PathBuf::from(arg);

    let list: Vec<String> = serde_json::from_slice(&std::fs::read(&list_json)?)?;
    for user_repo in &list {
        repo::Repo::new(user_repo)?.output()?;
    }

    Ok(())
}
