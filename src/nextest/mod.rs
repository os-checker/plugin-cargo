//! Ref: https://github.com/nextest-rs/nextest/blob/cb67e450e0fa2803f0089ffc9189c34ecd355f13/nextest-runner/src/reporter/structured/libtest.rs#L116
use std::hash::Hash;

use crate::prelude::*;
use indexmap::Equivalent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportTest {
    #[serde(rename = "type")]
    typ: TypeTest,
    event: Event,
    name: Name,
    /// execution time in seconds; Some for Event::ok
    exec_time: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(try_from = "&str")]
pub struct TypeTest;

impl TryFrom<&'_ str> for TypeTest {
    type Error = &'static str;

    fn try_from(text: &'_ str) -> Result<Self, Self::Error> {
        if text == "test" {
            Ok(Self)
        } else {
            Err("only support test type")
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Event {
    Started,
    Ok,
    Failed,
    Ignored,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(from = "&str")]
pub struct Name {
    pkg_name: String,
    test_binary: String,
    test_case: String,
}

impl Hash for Name {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        [&*self.pkg_name, &*self.test_binary, &*self.test_case].hash(state);
    }
}

impl Equivalent<Name> for [&'_ str; 3] {
    fn equivalent(&self, name: &Name) -> bool {
        let [pkg_name, test_binary, test_case] = *self;
        name.test_case == test_case && name.test_binary == test_binary && name.pkg_name == pkg_name
    }
}

// pkg-name::test_binary_name$testcase_path#n
// #n is an optional suffix if the test was retried for n times (ignored for now)
impl From<&'_ str> for Name {
    fn from(mut text: &'_ str) -> Self {
        let pkg_name_end = text.find(':').unwrap();
        let pkg_name = text[..pkg_name_end].to_owned();

        text = &text[pkg_name_end + 2..];

        let test_binary_end = text.find('$').unwrap();
        let test_binary = text[..test_binary_end].to_owned();

        text = &text[test_binary_end + 1..];

        let test_case_end = text.find('#').unwrap_or(text.len());
        let test_case = text[..test_case_end].to_owned();

        Name {
            pkg_name,
            test_binary,
            test_case,
        }
    }
}

#[test]
fn string_to_name() {
    let text = "os-checker-plugin-cargo::os_checker_plugin_cargo$repo::test_cargo_tomls";
    let name = Name::from(text);
    dbg!(&name);

    let text_retry = "os-checker-plugin-cargo::os_checker_plugin_cargo$repo::test_cargo_tomls#2";
    let name = Name::from(text_retry);
    dbg!(&name);
}

#[test]
fn parse_test_event() {
    let text = r#"{"type":"test","event":"started","name":"os-checker-plugin-cargo::t1$from_t1"}"#;
    let report: ReportTest = serde_json::from_str(text).unwrap();
    dbg!(report);
}

#[test]
fn parse_stream() {
    let text = std::fs::read_to_string("tests/nextest.stdout").unwrap();
    let reports = parse_test_reports(&text);
    dbg!(&reports);
    assert!(!reports.is_empty());
}

fn parse_test_reports(text: &str) -> Vec<ReportTest> {
    text.lines()
        .filter_map(|line| serde_json::from_str::<ReportTest>(line).ok())
        .collect()
}

pub fn run_testcases() -> Result<(String, IndexMap<Name, (Event, Option<f32>)>)> {
    let output = duct::cmd!(
        "cargo",
        "nextest",
        "run",
        "--message-format",
        "libtest-json-plus"
    )
    .env("NEXTEST_EXPERIMENTAL_LIBTEST_JSON", "1")
    .stdout_capture()
    .stderr_capture()
    .unchecked()
    .run()?;

    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    let reports = parse_test_reports(std::str::from_utf8(&output.stdout)?);
    // new event overrides old ones:
    // e.g. if a test result is ok, we won't get its started report
    let testcases = reports
        .into_iter()
        .map(|report| (report.name, (report.event, report.exec_time)))
        .collect();

    Ok((stderr, testcases))
}

// NEXTEST_EXPERIMENTAL_LIBTEST_JSON=1 cargo nextest run --message-format libtest-json-plus
#[test]
fn run_and_parse() -> Result<()> {
    // Why doesn't this cause infinite test running?
    let (stderr, testcases) = run_testcases()?;

    println!("stderr={stderr}\ntestcases={testcases:?}");

    let got = testcases.get(&[
        "os-checker-plugin-cargo",
        "os_checker_plugin_cargo",
        "nextest::parse_stream",
    ]);
    assert!(got.is_some());

    Ok(())
}
