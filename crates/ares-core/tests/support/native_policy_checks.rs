//! Missing-dependency policy probes use real integration-test subprocess entrypoints.
pub fn require_missing_dependencies_fail(selector: &str, keys: &[&str]) {
    let artifacts = std::env::temp_dir().join(format!(
        "ares-native-policy-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&artifacts).unwrap();
    for key in keys {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", selector, "--nocapture"])
            .env("ARES_REQUIRE_ORCA", "1")
            .env("ARES_ROLE_ORACLE_LIVE", "1")
            .env_remove(key)
            .output()
            .unwrap();
        let diagnostic = format!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        std::fs::write(artifacts.join(format!("{key}.log")), &diagnostic).unwrap();
        assert!(
            diagnostic.contains("running 1 test"),
            "selector drift: {diagnostic}"
        );
        assert!(
            !output.status.success() && diagnostic.contains(key),
            "missing {key} falsely accepted or lost cause; artifacts {artifacts:?}: {diagnostic}"
        );
    }
}
