use os_checker_types::Utf8Path;

use child_wait_timeout::ChildWT;
use std::io::Read;
use std::process::{Command, Stdio};
use std::time::Duration;

pub fn cargo_miri(
    pkg: &str,
    kind: &str,
    bin: &str,
    name: &str,
    workspace_root: &Utf8Path,
) -> Option<String> {
    let kind = format!("--{kind}");
    let cmd = format!("cargo miri test -p {pkg} {kind} {bin} -- {name}");
    let _span = error_span!("miri", cmd).entered();

    let mut child = Command::new("cargo")
        .args(["miri", "test", "-p", pkg, &kind, bin, "--", name])
        .stderr(Stdio::piped())
        .current_dir(workspace_root)
        .spawn()
        .map_err(|err| error!("Failed to spawn miri command: {err}"))
        .ok()?;

    let minutes = 2;
    let success = match child.wait_timeout(Duration::from_secs(minutes * 60)) {
        // Finished in 2 minutes
        Ok(status) => status.success(),
        Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
            let err = format!("Process timed out for {minutes} minutes.");
            error!(err);
            return Some(format!("cmd={cmd}\n{err}"));
        }
        Err(e) => {
            let err = format!("Failed to wait on process: {e:?}");
            error!(err);
            return Some(format!("cmd={cmd}\n{err}"));
        }
    };

    if success {
        // stderr may contain compilation information like
        // stderr="    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s\n
        // Running unittests src/lib.rs (target/miri/x86_64-unknown-linux-gnu/debug/deps/os_checker_plugin_cargo-457c2a400d4e8077)\n"
        return None;
    }

    let Some(mut stderr_child) = child.stderr else {
        error!("Child stderr is unavailable.");
        return None;
    };

    let mut stderr_buf = Vec::with_capacity(1024);
    if let Err(err) = stderr_child.read_to_end(&mut stderr_buf) {
        error!("Failed to read stderr: {err}");
        return None;
    };

    let stderr = String::from_utf8(strip_ansi_escapes::strip(stderr_buf))
        .map_err(|err| error!("{err}: Non-utf8 output is emitted."))
        .ok()?;

    Some(stderr)
}

#[test]
fn miri_output() {
    let stderr = cargo_miri(
        "os-checker-plugin-cargo",
        "test",
        "t1",
        "miri_should_err",
        ".".into(),
    )
    .unwrap();
    eprintln!("{stderr}");
}
