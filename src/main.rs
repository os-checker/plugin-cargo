mod logger;
mod repo;

mod prelude {
    pub use camino::{Utf8Path, Utf8PathBuf};
    pub use eyre::{Context, Result};
}

use prelude::*;

fn main() -> Result<()> {
    logger::init();

    Ok(())
}
