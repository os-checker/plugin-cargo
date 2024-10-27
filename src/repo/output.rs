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
    pub author: Vec<String>,
    pub description: String,
    pub categories: Vec<String>,
    pub os_categories: Vec<String>,
}

impl Output {
    pub fn new(pkg: &Package, testcases: Option<TestCases>) -> Self {
        Output {
            version: pkg.version.to_string(),
            testcases,
            dependencies: pkg.dependencies.len(),
            lib: pkg.targets.iter().any(|t| t.is_lib()),
            bin: pkg.targets.iter().any(|t| t.is_bin()),
            tests: pkg.targets.iter().map(|t| t.is_test()).count(),
            examples: pkg.targets.iter().map(|t| t.is_example()).count(),
            benches: pkg.targets.iter().map(|t| t.is_bench()).count(),
            author: pkg.authors.clone(),
            description: pkg.description.clone().unwrap_or_default(),
            categories: pkg.categories.clone(),
            os_categories: pkg
                .metadata
                .get("os")
                .and_then(|os| {
                    Some(
                        os.get("categories")?
                            .as_array()?
                            .iter()
                            .filter_map(|x| x.as_str().map(String::from))
                            .collect(),
                    )
                })
                .unwrap_or_default(),
        }
    }
}
