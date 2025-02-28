use os_checker_plugin_cargo::{repo::write_output_json, BASE_DIR};
use plugin::{logger, prelude::*, repos, write_json};

#[macro_use]
extern crate tracing;

mod cache;

fn main() -> Result<()> {
    logger::init();

    let list = repos()?;
    let mut outputs = Vec::with_capacity(list.len());

    for user_repo in &list {
        let _span = error_span!("list", user_repo).entered();
        match cache::get_or_gen_cache(user_repo) {
            Ok((key, val)) => {
                let json = val.into_json();
                write_output_json(&key.user, &key.repo, &json)?;
                outputs.push(json);
            }
            Err(err) => error!(?err),
        };
    }

    write_json(
        &Utf8PathBuf::from_iter([BASE_DIR, "summaries.json"]),
        &outputs,
    )?;

    Ok(())
}
