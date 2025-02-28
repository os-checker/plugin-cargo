use crate::Result;
use os_checker_plugin_cargo::repo::{split_user_repo, GitInfo, Repo};
use plugin::prelude::{cmd, serde_json};
use redb::{Database, Error, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};

const TABLE: TableDefinition<CachedKey, CachedValue> = TableDefinition::new("plugin-cargo");

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CachedKey {
    user: String,
    repo: String,
    api: Api,
}

impl redb::Key for CachedKey {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}
impl redb::Value for CachedKey {
    type SelfType<'a>
        = CachedKey
    where
        Self: 'a;

    type AsBytes<'a>
        = Vec<u8>
    where
        Self: 'a;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        serde_json::from_slice(data).expect("Failed to deserialize CachedKey from bytes.")
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'b,
    {
        serde_json::to_vec(value).expect("Failed to serialize CachedKey into bytes.")
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("[plugin-cargo] CachedKey")
    }
}

#[derive(Debug)]
pub struct CachedValue {
    inner: serde_json::Value,
}

impl CachedValue {
    pub fn json(&self) -> &serde_json::Value {
        &self.inner
    }
}

impl redb::Value for CachedValue {
    type SelfType<'a>
        = Self
    where
        Self: 'a;

    type AsBytes<'a>
        = Vec<u8>
    where
        Self: 'a;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        Self {
            inner: serde_json::from_slice(data)
                .expect("Failed to deserialize CachedValue from bytes."),
        }
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'b,
    {
        serde_json::to_vec(&value.inner).expect("Failed to serialize CachedValue into bytes.")
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("[plugin-cargo] CachedValue")
    }
}

/// Generate a new cached repo and its output regarding tests and package information.
pub fn gen_cache(user_repo: &str) -> Result<(CachedKey, CachedValue)> {
    let repo = Repo::new(user_repo)?;
    let output = repo.output()?;
    std::fs::remove_dir_all(&repo.dir)?;
    Ok((
        CachedKey {
            user: repo.user,
            repo: repo.repo,
            api: repo.git_info.into(),
        },
        CachedValue { inner: output },
    ))
}

pub fn get_or_gen(user_repo: &str) -> Result<(CachedKey, CachedValue)> {
    let key = github_graphql_api(user_repo)?;
    let (key, val) = match read_cache(&key)? {
        Some(val) => (key, val),
        None => gen_cache(user_repo)?,
    };
    Ok((key, val))
}

const FILE: &str = "cache-plugin-cargo-v0.1.4.redb";

fn read_cache(key: &CachedKey) -> Result<Option<CachedValue>> {
    let db = Database::create(FILE)?;
    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(TABLE)?;
    Ok(table.get(key)?.map(|val| val.value()))
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Api {
    branch: String,
    sha: String,
}

impl From<GitInfo> for Api {
    fn from(info: GitInfo) -> Self {
        Api {
            branch: info.branch,
            sha: info.sha,
        }
    }
}

fn github_graphql_api(user_repo: &str) -> Result<CachedKey> {
    // gh api graphql -f query='
    //   query($owner: String!, $name: String!) {
    //     repository(owner: $owner, name: $name) {
    //       defaultBranchRef {
    //         name
    //         target {
    //           ... on Commit {
    //             oid
    //           }
    //         }
    //       }
    //     }
    //   }
    // ' -F owner="os-checker" -F name="os-checker-test-suite" |
    //   jq ".data.repository.defaultBranchRef | {branch: .name, sha: .target.oid}"
    const QUERY: &str = "query=
query($owner: String!, $name: String!) {
  repository(owner: $owner, name: $name) {
    defaultBranchRef {
      name
      target {
        ... on Commit {
          oid
        }
      }
    }
  }
}";

    let [user, repo] = split_user_repo(user_repo)?;
    let owner = format!("owner={user}");
    let name = format!("name={repo}");
    let expr = cmd!("gh", "api", "graphql", "-f", QUERY, "-F", owner, "-F", name).pipe(cmd!(
        "jq",
        ".data.repository.defaultBranchRef | {branch: .name, sha: .target.oid}"
    ));
    let json = expr.read()?;
    let api: Api = serde_json::from_str(&json)?;
    Ok(CachedKey { user, repo, api })
}

#[test]
fn test_os_checker_test_suite() -> Result<()> {
    const FILE: &str = "cache-plugin-cargo-v-test.redb";

    let (key, val) = gen_cache("os-checker/os-checker-test-suite")?;

    let db = Database::create(FILE)?;

    let write_txn = db.begin_write()?;
    {
        let mut table = write_txn.open_table(TABLE)?;
        table.insert(&key, &val)?;
    }
    write_txn.commit()?;

    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(TABLE)?;
    assert_eq!(val.json(), table.get(&key)?.unwrap().value().json());

    Ok(())
}

#[test]
fn test_github_graphql_api() -> Result<()> {
    dbg!(github_graphql_api("os-checker/os-checker-test-suite")?);
    Ok(())
}
