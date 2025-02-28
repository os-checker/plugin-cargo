use crate::Result;
use os_checker_plugin_cargo::repo::Repo;

mod types;
pub use types::{Api, CachedKey, CachedValue};

mod db;
mod gh;

/// Generate a new cached repo and its output regarding tests and package information.
fn gen_cache(user_repo: &str) -> Result<(CachedKey, CachedValue)> {
    let repo = Repo::new(user_repo)?;
    let output = repo.output()?;

    // remove local dir: all local operations must take place before this
    std::fs::remove_dir_all(&repo.dir)?;

    Ok((
        CachedKey {
            user: repo.user,
            repo: repo.repo,
            api: repo.git_info.into(),
        },
        CachedValue::new(output),
    ))
}

/// Get a local cache if any, otherwise download the repo and generate the cache.
pub fn get_or_gen_cache(user_repo: &str) -> Result<(CachedKey, CachedValue)> {
    let key = gh::graphql_api(user_repo)?;
    let db = db::Db::open()?;
    let (key, val) = match db.load_cache(&key)? {
        Some(val) => (key, val),
        None => gen_cache(user_repo)?,
    };
    db.store_cache(&key, &val)?;
    Ok((key, val))
}
