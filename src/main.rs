mod prelude {
    pub use camino::{Utf8Path, Utf8PathBuf};
    pub use cargo_metadata::Metadata;
    pub use eyre::{Context, Result};
    pub use indexmap::IndexMap;
}

use prelude::*;

#[macro_use]
extern crate eyre;

mod logger;
mod repo;

fn main() -> Result<()> {
    logger::init();

    Ok(())
}
