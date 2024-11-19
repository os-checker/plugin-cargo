use crate::prelude::*;
use indexmap::Equivalent;
use serde::Deserialize;
use std::{hash::Hash, sync::LazyLock};

/// Gross diagnostics amount on all targets for each package.
const URL: &str = "https://raw.githubusercontent.com/os-checker/database/refs/heads/main/ui/home/split/All-Targets.json";

#[derive(Debug, Deserialize)]
pub struct Item {
    children: Vec<Child>,
}

#[derive(Debug, Deserialize)]
pub struct Child {
    data: Data,
}

#[derive(Debug, Deserialize)]
pub struct Data {
    user: String,
    repo: String,
    pkg: String,
    total_count: usize,
}

#[derive(Debug, Deserialize)]
#[serde(from = "Vec<Item>")]
pub struct DiagnosticsCount {
    map: IndexMap<Key, usize>,
}

impl DiagnosticsCount {
    fn new() -> Result<Self> {
        let json = duct::cmd!("wget", URL, "-O", "-").read()?;
        Ok(serde_json::from_str(&json)?)
    }
}

impl From<Vec<Item>> for DiagnosticsCount {
    fn from(value: Vec<Item>) -> Self {
        DiagnosticsCount {
            map: value
                .into_iter()
                .flat_map(|val| {
                    val.children.into_iter().map(|child| {
                        let Data {
                            user,
                            repo,
                            pkg,
                            total_count,
                        } = child.data;
                        (Key { user, repo, pkg }, total_count)
                    })
                })
                .collect(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Key {
    user: String,
    repo: String,
    pkg: String,
}

impl Hash for Key {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        [&*self.user, &*self.repo, &*self.pkg].hash(state);
    }
}

impl Equivalent<Key> for [&'_ str; 3] {
    fn equivalent(&self, key: &Key) -> bool {
        let &[user, repo, pkg] = self;
        user == key.user && repo == key.repo && pkg == key.pkg
    }
}

pub static DIAGNOSTICS_COUNT: LazyLock<DiagnosticsCount> =
    LazyLock::new(|| DiagnosticsCount::new().unwrap());

#[test]
fn test_diagnostics_count() {
    dbg!(&*DIAGNOSTICS_COUNT);
}

pub fn diag_total_count(key: [&str; 3]) -> Option<usize> {
    DIAGNOSTICS_COUNT.map.get(&key).copied()
}
