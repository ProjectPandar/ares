use super::super::{OrcaRunner, cache_identity, environment, repo_root};
use std::{fs, path::PathBuf, process::Command};

pub(super) fn runner(test: &str) -> Option<OrcaRunner> {
    let (_, root) = environment::available().unwrap()?;
    let bin = crate::native_environment::binary("ARES_KSR_INITIALIZED_BIN")?;
    let native = crate::native_environment::binary("ARES_KSR_INITIALIZED_NATIVE_BIN")?;
    let resources = PathBuf::from(crate::native_environment::value(
        "ARES_KSR_INITIALIZED_RESOURCES_DIR",
    )?);
    assert!(resources.is_dir(), "initialized resources {resources:?}");
    let launcher = repo_root().join("scripts/orca-seeded-parity.sh");
    let selected = [
        ("ARES_ORCA_BIN", launcher.as_os_str()),
        ("ARES_ORCA_NATIVE_BIN", native.as_os_str()),
        ("ARES_ORCA_RESOURCES_DIR", resources.as_os_str()),
        ("ORCA_REPLAY_BIN", bin.as_os_str()),
    ]
    .into_iter()
    .all(|(key, value)| std::env::var_os(key).as_deref() == Some(value))
        && std::env::var("ORCA_REPLAY_SEED").as_deref() == Ok("0");
    if !selected {
        let replay_cpu = std::env::var_os("ORCA_REPLAY_CPU").expect(
            "ORCA_REPLAY_CPU must be supplied by the outer runner to preserve lane affinity",
        );
        let artifacts = tempfile::Builder::new()
            .prefix("ksr-initialized-dispatch-")
            .tempdir_in(root)
            .unwrap()
            .keep();
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", test, "--nocapture"])
            .env("ARES_REQUIRE_ORCA", "1")
            .env("ARES_ORCA_BIN", &launcher)
            .env("ARES_ORCA_NATIVE_BIN", &native)
            .env("ARES_ORCA_RESOURCES_DIR", &resources)
            .env("ORCA_REPLAY_BIN", &bin)
            .env("ORCA_REPLAY_SEED", "0")
            .env("ORCA_REPLAY_CPU", replay_cpu)
            .output()
            .unwrap();
        fs::write(artifacts.join("child.stdout"), &output.stdout).unwrap();
        fs::write(artifacts.join("child.stderr"), &output.stderr).unwrap();
        let diagnostic = format!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            diagnostic.contains("running 1 test"),
            "{artifacts:?}: {diagnostic}"
        );
        assert!(output.status.success(), "{artifacts:?}: {diagnostic}");
        assert!(
            !diagnostic.contains("SKIPPED:"),
            "{artifacts:?}: {diagnostic}"
        );
        return None;
    }
    // 2.5.0 seed-replay producer identity (ard-ksr-initialized-runtime-authority.md).
    assert_eq!(
        cache_identity::raw_digest(&fs::read(&bin).unwrap()),
        "9d285c9995cdd0eacef354afb76853c81e3def3154b12e38a50238bf268004d6"
    );
    assert_eq!(
        cache_identity::raw_digest(&fs::read(&native).unwrap()),
        "84a7780653be3b46fe909e43ae98b3576907c561e8ff337c208ff97c2b23a84a"
    );
    OrcaRunner::from_env().unwrap()
}
