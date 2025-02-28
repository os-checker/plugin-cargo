use os_checker_plugin_cargo::repo::GitInfo;
use plugin::prelude::serde_json;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CachedKey {
    pub user: String,
    pub repo: String,
    pub api: Api,
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
    pub fn new(inner: serde_json::Value) -> Self {
        Self { inner }
    }

    pub fn into_json(self) -> serde_json::Value {
        self.inner
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Api {
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
