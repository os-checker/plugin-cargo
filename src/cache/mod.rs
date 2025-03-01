use crate::Result;
use os_checker_plugin_cargo::repo::Repo;
use plugin::prelude::serde_json;

mod types;
pub use types::{Api, CachedKey, CachedValue};

mod db;
mod gh;

/// Output json when error happens.
fn err_json(user: &str, repo: &str, err: &dyn std::error::Error) -> serde_json::Value {
    let msg = strip_ansi_escapes::strip_str(format!("{err}"));
    let now = os_checker_types::now();
    serde_json::json!({
        "user": user,
        "repo": repo,
        "timestamp": {
            "start": now,
            "end": now
        },
        "err": msg
    })
}

/// Generate a new cached repo and its output regarding tests and package information.
fn gen_cache(user_repo: &str) -> Result<(CachedKey, CachedValue)> {
    let repo = Repo::new(user_repo)?;
    let output = match repo.output() {
        Ok(output) => output,
        Err(err) => err_json(&repo.user, &repo.repo, err.as_ref()),
    };

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

fn gen_cache_consuming_error(user_repo: &str, key: CachedKey) -> (CachedKey, CachedValue) {
    match gen_cache(user_repo) {
        Ok(cache) => cache,
        Err(err) => {
            let val = CachedValue::new(err_json(&key.user, &key.repo, err.as_ref()));
            (key, val)
        }
    }
}

/// Get a local cache if any, otherwise download the repo and generate the cache.
pub fn get_or_gen_cache(user_repo: &str) -> Result<(CachedKey, CachedValue)> {
    let key = gh::graphql_api(user_repo)?;
    let _span = error_span!("cache", key = format!("{:?}", key.api)).entered();

    let db = db::Db::open()?;
    let force = std::env::var("OS_CHECKER_FORCE_PLUGIN_CARGO");
    let (key, mut val) = if let Ok("true") = force.as_deref() {
        gen_cache_consuming_error(user_repo, key)
    } else {
        match db.load_cache(&key)? {
            Some(val) => (key, val),
            None => gen_cache_consuming_error(user_repo, key),
        }
    };
    val.update_timestamp();
    db.store_cache(&key, &val)?;
    Ok((key, val))
}
