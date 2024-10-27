use cargo_metadata::Package;
use testcases::PkgTests;

use crate::prelude::*;

mod testcases;

#[derive(Debug)]
pub struct Repo {
    user: String,
    repo: String,
    // repo root
    dir: Utf8PathBuf,
    cargo_tomls: Vec<Utf8PathBuf>,
    workspaces: Workspaces,
}

impl Repo {
    pub fn new(user_repo: &str) -> Result<Repo> {
        let v: Vec<_> = user_repo.split("/").collect();
        let user = v[0].to_owned();
        let repo = v[1].to_owned();

        let dir = git_clone(&user, &repo)?;

        let mut cargo_tomls = get_cargo_tomls_recursively(&dir);
        cargo_tomls.sort_unstable();

        let workspaces = workspaces(&cargo_tomls)?;

        Ok(Repo {
            user,
            repo,
            dir,
            cargo_tomls,
            workspaces,
        })
    }

    // /// packages in all repos
    fn packages_dirs(&self) -> Vec<&Package> {
        self.workspaces
            .values()
            .map(|ws| ws.workspace_packages())
            .flatten()
            .collect()
    }

    fn get_pkg_tests(&self) -> Result<PkgTests> {
        let mut map = PkgTests::new();
        for workspace_root in self.workspaces.keys() {
            map.extend(testcases::get(&self.dir, workspace_root)?);
        }
        Ok(map)
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

type Workspaces = IndexMap<Utf8PathBuf, Metadata>;

fn workspaces(cargo_tomls: &[Utf8PathBuf]) -> Result<Workspaces> {
    let mut map = IndexMap::new();
    for cargo_toml in cargo_tomls {
        // NOTE: 一旦支持 features，这里可能需要传递它们
        let metadata = cargo_metadata::MetadataCommand::new()
            .manifest_path(cargo_toml)
            .exec()
            .map_err(|err| eyre!("无法读取 cargo metadata 的结果：{err}"))?;
        let root = &metadata.workspace_root;
        // 每个 member package 解析的 workspace_root 和 members 是一样的
        if !map.contains_key(root) {
            map.insert(root.clone(), metadata);
        }
    }
    map.sort_unstable_keys();
    Ok(map)
}

#[test]
fn test_cargo_tomls() {
    dbg!(get_cargo_tomls_recursively(Utf8Path::new(".")));
}
