use crate::{crates_io::IndexFile, database::diag_total_count};
use cargo_metadata::Package;
use eyre::ContextCompat;
use output::Output;
use plugin::{prelude::*, write_json};
use std::sync::LazyLock;
use testcases::PkgTests;

mod git_info;
pub use git_info::GitInfo;

mod miri;
mod os_checker;
mod output;
mod testcases;

pub fn split_user_repo(user_repo: &str) -> Result<[String; 2]> {
    let mut split = user_repo.split("/");
    let user = split
        .next()
        .with_context(|| format!("Not found user in `{user_repo}`."))?;
    let repo = split
        .next()
        .with_context(|| format!("Not found repo in `{user_repo}`."))?;
    Ok([user.to_owned(), repo.to_owned()])
}

#[derive(Debug)]
pub struct Repo {
    pub user: String,
    pub repo: String,
    // repo root
    pub dir: Utf8PathBuf,
    pub pkg_targets: os_checker::PkgTargets,
    pub cargo_tomls: Vec<Utf8PathBuf>,
    pub workspaces: Workspaces,
    pub git_info: GitInfo,
}

impl Repo {
    pub fn new(user_repo: &str) -> Result<Repo> {
        let [user, repo] = split_user_repo(user_repo)?;

        // this implies repo downloading
        let pkg_targets = os_checker::run(user_repo)?;

        let dir = local_repo_dir(&user, &repo);
        let mut cargo_tomls = get_cargo_tomls_recursively(&dir);
        cargo_tomls.sort_unstable();

        let workspaces = workspaces(&cargo_tomls)?;

        let git_info = GitInfo::new(&dir)?;

        Ok(Repo {
            user,
            repo,
            dir,
            pkg_targets,
            cargo_tomls,
            workspaces,
            git_info,
        })
    }

    // packages in all workspaces
    fn packages(&self) -> Vec<&Package> {
        self.workspaces
            .values()
            .flat_map(|ws| ws.workspace_packages())
            // but don't emit packages that are not checked by os-checker
            .filter(|pkg| self.pkg_targets.get(pkg.name.as_str()).is_some())
            .collect()
    }

    fn contains_x64(&self, pkg: &str) -> bool {
        if let Some(targets) = self.pkg_targets.get(pkg) {
            for target in targets {
                if target == "x86_64-unknown-linux-gnu" {
                    return true;
                }
            }
        }
        false
    }

    fn get_pkg_tests(&self) -> Result<PkgTests> {
        let mut map = PkgTests::new();
        for (workspace_root, meta) in &self.workspaces {
            // NOTE: nextest is run under all packages in a workspace,
            // maybe we should run tests for each package?
            // FIXME: since --target is not supported in nextest and miri yet,
            // we should tell them not run tests other than on x86_64-unknown-linux-gnu.
            'inner: for pkg in meta.workspace_packages() {
                if self.contains_x64(&pkg.name) {
                    map.extend(testcases::get(workspace_root)?);
                    break 'inner;
                }
            }
        }
        Ok(map)
    }

    pub fn output(&self) -> Result<serde_json::Value> {
        let mut test_cases = self
            .get_pkg_tests()
            .inspect_err(|err| error!(?err, "Failed to get testcases"))
            .unwrap_or_default();
        let pkgs = self.packages();

        let last_commit_time = self.git_info.last_commit.to_string();

        let mut outputs = IndexMap::with_capacity(pkgs.len());
        for pkg in pkgs {
            let pkg_name = pkg.name.as_str();
            let _span = error_span!("output", pkg = pkg_name).entered();

            let mut output = Output::new(pkg, test_cases.swap_remove(pkg_name), &last_commit_time);
            output.diag_total_count = diag_total_count([&self.user, &self.repo, pkg_name]);

            match IndexFile::new(pkg_name) {
                Ok(mut index_file) => {
                    output.release_count = Some(index_file.release_count());
                    match index_file.get_last_release_info() {
                        Ok(()) => {
                            if let Some((size, time)) = index_file.last_release_size_and_time() {
                                output.last_release_size = Some(size);
                                output.last_release_time = Some(time.to_string());
                            }
                        }
                        Err(err) => error!(?err),
                    }
                }
                Err(err) => error!(?err, "Unable to handle index file"),
            };

            assert!(
                outputs.insert(pkg_name, output).is_none(),
                "os-checker can't handle duplicated package names in a repo"
            );
        }

        outputs.sort_unstable_keys();

        let now = os_checker_types::now();
        let json = serde_json::json!({
            "user": self.user,
            "repo": self.repo,
            "timestamp": {
                "start": now,
                "end": now
            },
            "pkgs": outputs
        });

        Ok(json)
    }

    pub fn remove_local_dir(self) -> Result<()> {
        std::fs::remove_dir_all(&self.dir)?;
        Ok(())
    }
}

pub fn write_output_json(user: &str, repo: &str, json: &serde_json::Value) -> Result<()> {
    let mut path = Utf8PathBuf::from_iter([crate::BASE_DIR, user, repo]);
    path.set_extension("json");
    write_json(&path, json)
}

pub fn local_base_dir() -> &'static Utf8Path {
    static GIT_CLONE_DIR: LazyLock<Utf8PathBuf> = LazyLock::new(|| {
        let path = Utf8PathBuf::from_iter(["/tmp", "os-checker-plugin-cargo"]);
        if let Err(err) = std::fs::create_dir_all(&path) {
            error!(?err, ?path, "directory is not created");
        };
        path
    });

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
            .map_err(|err| {
                eyre!(
                    "无法读取 cargo metadata 的结果：\n{}",
                    strip_ansi_escapes::strip_str(format!("{err}"))
                )
            })?;
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
