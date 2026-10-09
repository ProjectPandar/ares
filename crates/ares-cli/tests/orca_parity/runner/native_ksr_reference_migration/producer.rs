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
            .env("ORCA_REPLAY_CPU", "0")
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
    // Historical producer identity verified by the serial fresh-capture preflight.
    assert_eq!(
        cache_identity::raw_digest(&fs::read(&bin).unwrap()),
        "3d05d4ccab740ca5e13aa94d0d00867a75f867ba2159b9f5b024dc0fcaa3bfaa"
    );
    assert_eq!(
        cache_identity::raw_digest(&fs::read(&native).unwrap()),
        "8f0cf69dd9494ccfe53bf6749d6b685ccb5a0dd0df206694ce50d421007cc03d"
    );
    OrcaRunner::from_env().unwrap()
}
