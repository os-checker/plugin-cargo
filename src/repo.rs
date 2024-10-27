use crate::prelude::*;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Repo {
    user: String,
    repo: String,
    // repo root
    dir: Utf8PathBuf,
    cargo_tomls: Vec<Utf8PathBuf>,
}

impl Repo {
    pub fn new(user_repo: &str) -> Result<Repo> {
        let v: Vec<_> = user_repo.split("/").collect();
        let user = v[0].to_owned();
        let repo = v[1].to_owned();

        let dir = git_clone(&user, &repo)?;
        let cargo_tomls = get_cargo_tomls_recursively(&dir);

        Ok(Repo {
            user,
            repo,
            dir,
            cargo_tomls,
        })
    }
}

pub fn git_clone(user: &str, repo: &str) -> Result<Utf8PathBuf> {
    let dir = Utf8PathBuf::from_path_buf(dirs::home_dir().unwrap())
        .unwrap()
        .join(user)
        .join(repo);
    std::fs::create_dir_all(&dir)?;

    let url = format!("https://github.com/{repo}.git");
    duct::cmd!("git", "clont", url, &dir)
        .run()
        .with_context(|| format!("fail to clone {repo}"))?;

    Ok(dir)
}

pub fn get_cargo_tomls_recursively(dir: &Utf8Path) -> Vec<Utf8PathBuf> {
    walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|entry| {
            if let Ok(e) = entry {
                if e.file_type().is_file() && e.file_name().to_str()? == "Cargo.toml" {
                    return Utf8PathBuf::from_path_buf(e.into_path())
                        .ok()?
                        .canonicalize_utf8()
                        .ok();
                }
            }
            None
        })
        .collect()
}

pub fn run_cargo_metadata(dir: &Utf8Path) {}

#[test]
fn test_cargo_tomls() {
    dbg!(get_cargo_tomls_recursively(Utf8Path::new(".")));
}
