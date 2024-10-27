mod prelude {
    pub use camino::{Utf8Path, Utf8PathBuf};
    pub use cargo_metadata::Metadata;
    pub use eyre::{Context, Result};
    pub use indexmap::IndexMap;
}

use prelude::*;

#[macro_use]
extern crate eyre;
#[macro_use]
extern crate tracing;

mod logger;
mod repo;

const BASE: &str = "tmp";

fn main() -> Result<()> {
    logger::init();

    let arg = std::env::args().nth(1);
    let list_json = Utf8PathBuf::from(arg.as_deref().unwrap_or("list.json"));

    let list: Vec<String> = serde_json::from_slice(&std::fs::read(&list_json)?)?;
    for user_repo in &list {
        let _span = error_span!("list", user_repo).entered();
        match repo::Repo::new(user_repo) {
            Ok(val) => match val.output() {
                Ok(_) => (),
                Err(err) => error!(?err),
            },
            Err(err) => error!(?err),
        };
    }

    Ok(())
}
