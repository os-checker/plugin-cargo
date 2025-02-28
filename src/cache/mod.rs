use crate::Result;
use os_checker_plugin_cargo::repo::{split_user_repo, Repo};
use plugin::prelude::{cmd, serde_json};
use redb::{Database, Error, ReadableTable, TableDefinition};

mod types;
pub use types::{Api, CachedKey, CachedValue};

const TABLE: TableDefinition<CachedKey, CachedValue> = TableDefinition::new("plugin-cargo");

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
        CachedValue::new(output),
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
