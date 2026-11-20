#![cfg(unix)]
use std::{fs, os::unix::fs::PermissionsExt, process::Command};

#[test]
fn wrapper_requires_explicit_reference_before_library_discovery() {
    let temp = tempfile::tempdir().unwrap();
    let cache = temp.path().join("cache");
    let output = Command::new("bash")
        .arg(super::repo_root().join("scripts/orca-parity.sh"))
        .arg("--help")
        .env_remove("ORCA_APPDIR")
        .env_remove("ARES_PARITY_ENV")
        .env("ORCA_LIB_CACHE", &cache)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(127));
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("ORCA_APPDIR must name an explicitly selected reference"),
        "{:?}",
        output
    );
    assert!(!cache.exists());
}

#[test]
fn wrapper_executes_only_the_selected_reference_with_exact_arguments() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("reference with spaces");
    fs::create_dir_all(root.join("bin")).unwrap();
    let binary = root.join("bin/orca-slicer");
    fs::write(
        &binary,
        "#!/bin/sh\nprintf '%s\\n' \"$APPDIR\" \"$LC_ALL\" \"$@\"\n",
    )
    .unwrap();
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();
    let cache = temp.path().join("cache");
    fs::write(&cache, temp.path().to_str().unwrap()).unwrap();
    let output = Command::new("bash")
        .arg(super::repo_root().join("scripts/orca-parity.sh"))
        .args(["--slice", "0", "model with spaces.3mf"])
        .env("ORCA_APPDIR", &root)
        .env("ORCA_LIB_CACHE", &cache)
        .env_remove("ORCA_LIB_REFRESH")
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("{}\nC\n--slice\n0\nmodel with spaces.3mf\n", root.display())
    );
}
