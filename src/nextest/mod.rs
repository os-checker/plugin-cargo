//! Ref: https://github.com/nextest-rs/nextest/blob/cb67e450e0fa2803f0089ffc9189c34ecd355f13/nextest-runner/src/reporter/structured/libtest.rs#L116
use indexmap::Equivalent;
use plugin::prelude::*;
use serde::{Deserialize, Deserializer, Serialize};
use std::hash::Hash;

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportTest {
    #[serde(rename = "type")]
    typ: TypeTest,
    event: Event,
    name: Name,
    /// execution time in seconds; Some for Event::ok
    exec_time: Option<f32>,
    /// running error: None means no error
    #[serde(default, deserialize_with = "strip_color")]
    stdout: Option<String>,
}

fn strip_color<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<String>::deserialize(deserializer)?.map(strip_ansi_escapes::strip_str))
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

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
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

pub fn run_testcases(ws_dir: &Utf8Path) -> Result<Report> {
    let output = duct::cmd!(
        "cargo",
        "nextest",
        "run",
        "--workspace",
        "--no-fail-fast",
        "--color=never",
        "--message-format",
        "libtest-json-plus"
    )
    .env("NEXTEST_EXPERIMENTAL_LIBTEST_JSON", "1")
    .stdout_capture()
    .stderr_capture()
    .unchecked()
    .dir(ws_dir)
    .run()?;

    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    let reports = parse_test_reports(std::str::from_utf8(&output.stdout)?);
    // new event overrides old ones:
    // e.g. if a test result is ok, we won't get its started report
    let testcases: IndexMap<_, _> = reports
        .into_iter()
        .map(|report| (report.name, (report.event, report.exec_time, report.stdout)))
        .collect();
    debug!(testcases.len = testcases.len());

    Ok(Report { stderr, testcases })
}

pub struct Report {
    pub stderr: String,
    /// FIXME: 尚未考虑 binary kind，也就是说，如果同名测试函数路径存在于 lib 和 bin，它们的数据不正确。
    /// 如果要知道 binary kind，需要解析 suite type 消息，并依赖于解析整个消息。
    /// 目前只读取 test type 消息，解析单个消息。
    pub testcases: IndexMap<Name, (Event, Option<f32>, Option<String>)>,
}

impl Report {
    pub fn get_test_case(
        &self,
        pkg_bin_test: &[&str; 3],
    ) -> (Option<Event>, Option<u32>, Option<String>) {
        match self.testcases.get(pkg_bin_test) {
            Some((e, t, stdout)) => (
                Some(*e),
                t.map(|f| (f * 1000.0).round() as u32),
                stdout.clone(),
            ), // second => millisecond
            None => (None, None, None),
        }
    }
}

// NEXTEST_EXPERIMENTAL_LIBTEST_JSON=1 cargo nextest run --message-format libtest-json-plus
#[test]
#[ignore = "manually trigger this to avoid recursion"]
fn run_and_parse() -> Result<()> {
    // Why doesn't this cause infinite test running?
    let Report {
        stderr: _,
        testcases,
    } = run_testcases(Utf8Path::new("."))?;

    // println!("stderr={stderr}\ntestcases={testcases:?}");
    // dbg!(&testcases);

    let got = testcases.get(&[
        "os-checker-plugin-cargo",
        "os_checker_plugin_cargo",
        "nextest::parse_stream",
    ]);
    assert!(got.is_some());
    dbg!(got);

    let got = testcases.get(&[
        "os-checker-plugin-cargo",
        "os_checker_plugin_cargo",
        "repo::os_checker::test_sel4",
    ]);
    assert!(got.is_some());
    dbg!(got);

    Ok(())
}
