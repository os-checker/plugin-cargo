use crate::nextest::{run_testcases, Event, Report};
use nextest_metadata::{RustTestSuiteSummary, TestListSummary};
use plugin::prelude::*;

fn test_list(dir: &Utf8Path) -> Result<TestListSummary> {
    let mut command = nextest_metadata::ListCommand::new();
    command.current_dir(dir);
    command
        .exec()
        .with_context(|| format!("fail to run `cargo nextest list` in {dir}"))
}

pub type PkgTests = IndexMap<String, TestCases>;

// FIXME: how should we handle doc tests?

// nextest reports all member tests even if it's run under a member, so we just run under workspace
pub fn get(workspace_root: &Utf8Path) -> Result<PkgTests> {
    let _span = error_span!("get_and_run", ?workspace_root).entered();

    info!("test_list starts");
    let summary = test_list(workspace_root).with_context(|| "failed to get test list")?;
    info!("run_testcases starts");
    let report = run_testcases(workspace_root).with_context(|| "failed to run tests")?;

    let workspace_tests_count = summary.test_count;
    // nextest will report all bins even if zero testcase, so don't show them
    if workspace_tests_count == 0 {
        return Ok(Default::default());
    }

    let mut map = PkgTests::with_capacity(summary.rust_suites.len());
    for ele in summary.rust_suites.values() {
        if ele.test_cases.is_empty() {
            // skip zero testcase
            continue;
        }

        let test = TestBinary::new(ele, &report);
        if let Some((_, _, tests)) = map.get_full_mut(&ele.package_name) {
            tests.tests.push(test);
        } else {
            let tests = TestCases {
                tests: vec![test],
                failed: 0,
                duration_ms: 0,
                pkg_tests_count: 0,
                workspace_tests_count,
            };
            map.insert(ele.package_name.clone(), tests);
        }
    }

    for ele in map.values_mut() {
        for t in &ele.tests {
            ele.failed += t.failed;
            ele.duration_ms += t.duration_ms;
            ele.pkg_tests_count += t.testcases.len();
        }
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
    pub failed: usize,
    pub duration_ms: usize,
    pub pkg_tests_count: usize,
    pub workspace_tests_count: usize,
}

#[derive(Debug, Serialize)]
pub struct TestBinary {
    pub id: String,
    pub kind: String,
    pub binary_name: String,
    // strip repo root
    // pub binary_path: String,
    pub testcases: Vec<TestCase>,
    /// how many testcases are failed
    pub failed: usize,
    /// total duration in ms; maybe zero for various reasons
    pub duration_ms: usize,
}

#[derive(Debug, Serialize)]
pub struct TestCase {
    name: String,
    status: Option<Event>,
    duration_ms: Option<u32>,
    error: Option<String>,
    miri: Option<String>,
}

impl TestCase {
    pub fn new(name: &str, pkg_name: &str, kind: &str, bin_name: &str, report: &Report) -> Self {
        let miri = super::miri::cargo_miri(pkg_name, kind, bin_name, name);
        let (status, duration_ms, error) = report.get_test_case(&[pkg_name, bin_name, name]);
        let name = name.to_owned();
        Self {
            name,
            status,
            duration_ms,
            error,
            miri,
        }
    }
}

impl TestBinary {
    pub fn new(ele: &RustTestSuiteSummary, report: &Report) -> Self {
        let binary = &ele.binary;
        let pkg_name = &*ele.package_name;
        let bin_name = &*binary.binary_name;
        let kind = &*binary.kind.0;
        let testcases: Vec<_> = ele
            .test_cases
            .keys()
            .map(|name| TestCase::new(name, pkg_name, kind, bin_name, report))
            .collect();
        let (failed, duration_ms) = testcases.iter().fold((0, 0), |(s, d), t| {
            let d = d + t.duration_ms.unwrap_or(0) as usize;
            if t.status == Some(Event::Failed) {
                (s + 1, d)
            } else {
                (s, d)
            }
        });
        TestBinary {
            id: binary.binary_id.to_string(),
            kind: binary.kind.to_string(),
            binary_name: bin_name.to_owned(),
            // binary_path: binary
            //     .binary_path
            //     .strip_prefix(repo_root)
            //     .unwrap_or(&binary.binary_path)
            //     .to_string(),
            testcases,
            failed,
            duration_ms,
        }
    }
}

#[test]
fn test_get_testcases() {
    plugin::logger::init();
    dbg!(get(".".into()).unwrap());
}
