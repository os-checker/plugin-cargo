use super::{Api, CachedKey, Result};
use os_checker_plugin_cargo::repo::split_user_repo;
use plugin::prelude::{cmd, serde_json};

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

/// gh api graphql -f query='
///   query($owner: String!, $name: String!) {
///     repository(owner: $owner, name: $name) {
///       defaultBranchRef {
///         name
///         target {
///           ... on Commit {
///             oid
///           }
///         }
///       }
///     }
///   }
/// ' -F owner="os-checker" -F name="os-checker-test-suite" |
///   jq ".data.repository.defaultBranchRef | {branch: .name, sha: .target.oid}"
pub fn graphql_api(user_repo: &str) -> Result<CachedKey> {
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
fn test_github_graphql_api() -> Result<()> {
    dbg!(graphql_api("os-checker/os-checker-test-suite")?);
    Ok(())
}
