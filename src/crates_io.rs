fn url(pkg: &str) -> String {
    const PREFIX: &str = "https://index.crates.io";

    // ref: https://doc.rust-lang.org/cargo/reference/registry-index.html#index-files
    let components = match pkg.len() {
        1 => &["1", pkg][..],
        2 => &["2", pkg],
        3 => {
            let (a, b) = pkg.split_at(1);
            &["3", a, b, pkg]
        }
        _ => {
            let (a, b) = pkg.split_at(2);
            let (b, _) = b.split_at(2);
            &[a, b, pkg]
        }
    };

    // e.g. https://raw.githubusercontent.com/rust-lang/crates.io-index/refs/heads/master/os/-c/os-checker
    let mut buf = String::with_capacity(128);
    buf.push_str(PREFIX);

    for c in components {
        buf.push('/');
        buf.push_str(c);
    }

    buf
}

/// NOTE: the result might be spurious due to network failure or invalid text;
///
/// None means no release found; 0 is an invalid value because there at least one
/// release if found.
pub fn get_release_count(pkg: &str) -> Option<usize> {
    let url = url(pkg);
    let output = duct::cmd!("wget", &url, "-O", "-")
        .stdout_capture()
        .stderr_null()
        .run()
        .ok()?;

    let text = std::str::from_utf8(&output.stdout).ok()?.trim();
    let count = text.lines().count();
    if count == 0 {
        error!(pkg, url, text, "count is an invalid value 0")
    }
    Some(count)
}

#[test]
fn test_get_release_count() {
    dbg!(get_release_count("os-checker"));
}
