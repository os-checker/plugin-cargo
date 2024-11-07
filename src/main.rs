use os_checker_plugin_cargo::{prelude::*, *};
use std::fs;

#[macro_use]
extern crate tracing;

fn main() -> Result<()> {
    logger::init();

    let arg = std::env::args().nth(1);
    let list_json = Utf8PathBuf::from(arg.as_deref().unwrap_or("list.json"));

    let list: Vec<String> = serde_json::from_slice(&fs::read(&list_json)?)?;
    let mut outputs = Vec::with_capacity(list.len());

    for user_repo in &list {
        let _span = error_span!("list", user_repo).entered();
        match repo::Repo::new(user_repo, repo::RepoSource::Github) {
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

#[test]
fn from_main() {}
