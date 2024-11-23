use os_checker_plugin_cargo::{repo, repos, write_json, BASE};
use plugin::{logger, prelude::*};

#[macro_use]
extern crate tracing;

fn main() -> Result<()> {
    logger::init();

    let list = repos()?;
    let mut outputs = Vec::with_capacity(list.len());

    for user_repo in &list {
        let _span = error_span!("list", user_repo).entered();
        match repo::Repo::new(user_repo) {
            Ok(repo) => {
                match repo.output() {
                    Ok(output) => outputs.push(output),
                    Err(err) => error!(?err),
                }
                if let Err(err) = repo.remove_local_dir() {
                    error!(?err);
                };
            }
            Err(err) => error!(?err),
        };
    }

    write_json(&Utf8PathBuf::from_iter([BASE, "summaries.json"]), &outputs)?;

    Ok(())
}
