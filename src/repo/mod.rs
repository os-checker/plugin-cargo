use crate::prelude::*;
use cargo_metadata::Package;
use eyre::ContextCompat;
use output::Output;
use std::sync::LazyLock;
use testcases::PkgTests;

mod os_checker;
mod output;
mod testcases;

#[derive(Debug)]
pub struct Repo {
    pub user: String,
    pub repo: String,
    // repo root
    pub dir: Utf8PathBuf,
    pub pkg_targets: os_checker::PkgTargets,
    pub cargo_tomls: Vec<Utf8PathBuf>,
    pub workspaces: Workspaces,
}

impl Repo {
    pub fn new(user_repo: &str) -> Result<Repo> {
        let mut split = user_repo.split("/");
        let user = split
            .next()
            .with_context(|| format!("Not found user in `{user_repo}`."))?
            .to_owned();
        let repo = split
            .next()
            .with_context(|| format!("Not found repo in `{user_repo}`."))?
            .to_owned();

        // this implies repo downloading
        let pkg_targets = os_checker::run(user_repo)?;

        let dir = local_repo_dir(&user, &repo);
        let mut cargo_tomls = get_cargo_tomls_recursively(&dir);
        cargo_tomls.sort_unstable();
        info!(?cargo_tomls);

        let workspaces = workspaces(&cargo_tomls)?;

        info!(?pkg_targets);
        Ok(Repo {
            user,
            repo,
            dir,
            pkg_targets,
            cargo_tomls,
            workspaces,
        })
    }

    // packages in all workspaces
    fn packages(&self) -> Vec<&Package> {
        self.workspaces
            .values()
            .flat_map(|ws| ws.workspace_packages())
            // but don't emit packages that are not checked by os-checker
            .filter(|pkg| {
                info!(pkg.name);
                self.pkg_targets.contains_key(pkg.name.as_str())
            })
            .collect()
    }

    fn get_pkg_tests(&self) -> Result<PkgTests> {
        let mut map = PkgTests::new();
        for workspace_root in self.workspaces.keys() {
            // NOTE: nextest is run under all packages in a workspace,
            // maybe we should run tests for each package?
            map.extend(testcases::get(&self.dir, workspace_root)?);
        }
        Ok(map)
    }

    pub fn output(&self) -> Result<serde_json::Value> {
        let mut test_cases = self
            .get_pkg_tests()
            .inspect_err(|err| error!(?err, "Failed to get testcases"))
            .unwrap_or_default();
        let pkgs = self.packages();
        info!(pkgs = ?pkgs.iter().map(|p| &p.name).collect::<Vec<_>>());

        let mut outputs = IndexMap::with_capacity(pkgs.len());
        for pkg in pkgs {
            let pkg_name = pkg.name.as_str();
            let output = Output::new(pkg, test_cases.swap_remove(pkg_name));
            assert!(
                outputs.insert(pkg_name, output).is_none(),
                "os-checker can't handle duplicated package names in a repo"
            );
        }

        outputs.sort_unstable_keys();

        let json = serde_json::json!({
            "user": self.user,
            "repo": self.repo,
            "pkgs": outputs
        });
        self.write_json(&json)?;

        Ok(json)
    }

    fn write_json(&self, json: &serde_json::Value) -> Result<()> {
        let mut path = Utf8PathBuf::from_iter([crate::BASE, &self.user, &self.repo]);
        path.set_extension("json");
        crate::write_json(&path, json)
    }

    pub fn remove_local_dir(self) -> Result<()> {
        std::fs::remove_dir_all(&self.dir)?;
        Ok(())
    }
}

pub fn local_base_dir() -> &'static Utf8Path {
    static GIT_CLONE_DIR: LazyLock<Utf8PathBuf> =
        LazyLock::new(|| Utf8PathBuf::from_iter(["/tmp", "os-checker-plugin-cargo"]));

    &GIT_CLONE_DIR
}

// dependes on where does os-checker put the repo
pub fn local_repo_dir(user: &str, repo: &str) -> Utf8PathBuf {
    let mut dir = local_base_dir().to_owned();
    dir.extend([user, repo]);
    dir
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

#[test]
fn test_pkg_targets() -> Result<()> {
    let repo = Repo::new("seL4/rust-sel4")?;
    dbg!(&repo.pkg_targets);
    repo.remove_local_dir()?;
    Ok(())
}
