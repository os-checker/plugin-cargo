use crate::prelude::*;
use nextest_metadata::{RustTestSuiteSummary, TestListSummary};
use serde::Serialize;

fn test_list(dir: &Utf8Path) -> Result<TestListSummary> {
    let mut command = nextest_metadata::ListCommand::new();
    command.current_dir(dir);
    command
        .exec()
        .with_context(|| format!("fail to run `cargo nextest list` in {dir}"))
}

pub type PkgTests = IndexMap<String, TestCases>;

// nextest reports all member tests even if it's run under a member, so we just run under workspace
pub fn get(repo_root: &Utf8Path, workspace_root: &Utf8Path) -> Result<PkgTests> {
    let summary = test_list(workspace_root)?;
    let workspace_tests_count = summary.test_count;

    let mut map = PkgTests::with_capacity(summary.rust_suites.len());
    for ele in summary.rust_suites.values() {
        let test = TestBinary::new(ele, repo_root);
        if let Some((_, _, tests)) = map.get_full_mut(&ele.package_name) {
            tests.tests.push(test);
        } else {
            let tests = TestCases {
                tests: vec![test],
                pkg_tests_count: 0,
                workspace_tests_count,
            };
            map.insert(ele.package_name.clone(), tests);
        }
    }

    for ele in map.values_mut() {
        ele.pkg_tests_count = ele.tests.iter().map(|t| t.testcases.len()).sum();
    }

    let sum_pkg_tests_count: usize = map.values().map(|p| p.pkg_tests_count).sum();
    ensure!(
        sum_pkg_tests_count == workspace_tests_count,
        "test cases count are not equal: sum_pkg_tests_count ({sum_pkg_tests_count}) \
         ≠ workspace_tests_count ({workspace_tests_count})"
    );

    Ok(map)
}

#[derive(Debug, Serialize)]
pub struct TestCases {
    pub tests: Vec<TestBinary>,
    pub pkg_tests_count: usize,
    pub workspace_tests_count: usize,
}

#[derive(Debug, Serialize)]
pub struct TestBinary {
    pub id: String,
    pub kind: String,
    pub binary_name: String,
    // strip repo root
    pub binary_path: String,
    pub testcases: Vec<String>,
}

impl TestBinary {
    pub fn new(ele: &RustTestSuiteSummary, repo_root: &Utf8Path) -> Self {
        let binary = &ele.binary;
        TestBinary {
            id: binary.binary_id.to_string(),
            kind: binary.kind.to_string(),
            binary_name: binary.binary_name.clone(),
            binary_path: binary
                .binary_path
                .strip_prefix(repo_root)
                .unwrap_or(&binary.binary_path)
                .to_string(),
            testcases: ele.test_cases.keys().map(|k| k.to_owned()).collect(),
        }
    }
}
