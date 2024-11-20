use super::testcases::TestCases;
use cargo_metadata::Package;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Output {
    pub version: String,
    pub dependencies: usize,
    pub lib: bool,
    pub bin: bool,
    pub testcases: Option<TestCases>,
    pub tests: usize,
    pub examples: usize,
    pub benches: usize,
    pub authors: Vec<String>,
    pub description: String,
    pub documentation: Option<String>,
    pub readme: Option<String>,
    pub homepage: Option<String>,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub rust_version: Option<String>,
    pub diag_total_count: Option<usize>,
    pub last_commit_time: String,
    /// crates.io 发版次数
    pub release_count: Option<usize>,
    pub last_release_size: Option<u64>,
    pub last_release_time: Option<String>,
}

impl Output {
    pub fn new(pkg: &Package, testcases: Option<TestCases>, last_commit_time: &str) -> Self {
        Output {
            version: pkg.version.to_string(),
            testcases,
            dependencies: pkg.dependencies.len(),
            lib: pkg.targets.iter().any(|t| t.is_lib()),
            bin: pkg.targets.iter().any(|t| t.is_bin()),
            tests: pkg.targets.iter().filter(|t| t.is_test()).count(),
            examples: pkg.targets.iter().filter(|t| t.is_example()).count(),
            benches: pkg.targets.iter().filter(|t| t.is_bench()).count(),
            authors: pkg.authors.clone(),
            description: pkg.description.clone().unwrap_or_default(),
            documentation: pkg.documentation.clone(),
            readme: pkg.readme.as_deref().map(|p| p.to_string()),
            homepage: pkg.homepage.clone(),
            keywords: pkg.keywords.clone(),
            categories: pkg.categories.clone(),
            rust_version: pkg.rust_version.clone().map(|v| v.to_string()),
            diag_total_count: None,
            last_commit_time: last_commit_time.to_owned(),
            release_count: None,
            last_release_size: None,
            last_release_time: None,
        }
    }
}
