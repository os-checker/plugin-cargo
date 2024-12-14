use plugin::prelude::duct::cmd;

pub fn cargo_miri(pkg: &str, kind: &str, bin: &str, name: &str) -> Option<String> {
    let _span = error_span!("miri", "cargo miri test -p {pkg} --{kind} {bin} -- {name}").entered();

    let kind = format!("--{kind}");
    info!("cargo miri test -p {pkg} {kind} {bin} -- {name}");
    let output = cmd!("cargo", "miri", "test", "-p", pkg, kind, bin, "--", name)
        .stderr_capture()
        .unchecked()
        .run()
        .map_err(|err| error!(?err))
        .ok()?;

    let stderr = String::from_utf8(strip_ansi_escapes::strip(output.stderr))
        .map_err(|err| error!("{err}: Non-utf8 output is emitted."))
        .ok()?;

    if output.status.success() {
        // stderr may contain compilation information like
        // stderr="    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s\n
        // Running unittests src/lib.rs (target/miri/x86_64-unknown-linux-gnu/debug/deps/os_checker_plugin_cargo-457c2a400d4e8077)\n"
        return None;
    }

    Some(stderr)
}

#[test]
fn miri_output() {
    let stderr = cargo_miri("os-checker-plugin-cargo", "test", "t1", "miri_should_err").unwrap();
    eprintln!("{stderr}");
}
