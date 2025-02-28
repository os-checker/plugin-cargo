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

pub fn get_or_gen_cache(user_repo: &str) -> Result<(CachedKey, CachedValue)> {
    let key = gh::graphql_api(user_repo)?;
    let (key, val) = match db::read_cache(&key)? {
        Some(val) => (key, val),
        None => gen_cache(user_repo)?,
    };
    Ok((key, val))
}
