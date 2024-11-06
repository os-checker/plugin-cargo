//! Ref: https://github.com/nextest-rs/nextest/blob/cb67e450e0fa2803f0089ffc9189c34ecd355f13/nextest-runner/src/reporter/structured/libtest.rs#L116
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(from = "&str")]
pub struct Name {
    pkg_name: String,
    test_binary: String,
    test_case: String,
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
    let reports: Vec<_> = text
        .lines()
        .filter_map(|line| serde_json::from_str::<ReportTest>(line).ok())
        .collect();
    dbg!(&reports);
    assert!(reports.len() > 0);
}
