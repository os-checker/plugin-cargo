use super::testcases::TestCases;
use cargo_metadata::Package;

pub struct Output {
    user: String,
    repo: String,
    version: String,
    dependencies: usize,
    lib: bool,
    bin: bool,
    testcases: Option<TestCases>,
    tests: usize,
    examples: usize,
    benches: usize,
    author: Vec<String>,
    description: String,
    categories: Vec<String>,
    os_categories: Vec<String>,
}

impl Output {
    pub fn new(pkg: &Package, testcases: Option<TestCases>, user: &str, repo: &str) -> Self {
        Output {
            user: user.to_owned(),
            repo: repo.to_owned(),
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
