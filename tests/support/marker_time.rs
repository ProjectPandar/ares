//! Test-only live source-kernel authority; never reads committed Native outputs.
#[path = "native_environment.rs"]
mod native_environment;

use std::{path::PathBuf, process::Command};

pub fn raw_fixture() -> Option<PathBuf> {
    native_environment::binary("ARES_MARKER_TIME_BIN")?;
    match Command::new("node").arg("--version").output() {
        Ok(output) => assert!(output.status.success(), "node --version failed"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            assert!(
                !native_environment::required(),
                "ARES_REQUIRE_ORCA=1: missing node; Native comparison not executed"
            );
            native_environment::skipped("node is unavailable");
            return None;
        }
        Err(error) => panic!("cannot launch node: {error}"),
    }
    let directory = tempfile::Builder::new()
        .prefix("marker-time-live-")
        .tempdir()
        .unwrap()
        .keep();
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new("node")
        .arg(repo.join("tools/chain-probe/marker-time-fixtures.mjs"))
        .arg(&directory)
        .output()
        .unwrap();
    std::fs::write(directory.join("generator.stdout"), &output.stdout).unwrap();
    std::fs::write(directory.join("generator.stderr"), &output.stderr).unwrap();
    assert!(
        output.status.success(),
        "live marker-time generator failed: {} (artifacts {directory:?})",
        String::from_utf8_lossy(&output.stderr)
    );
    Some(directory)
}
