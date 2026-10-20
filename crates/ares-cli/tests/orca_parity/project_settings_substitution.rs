//! Config.cpp:657-676 / Config.hpp:2032-2040: EnableSilent enum substitution.
use crate::{
    OrcaRunner,
    runner::native_api_cli::{NativeCase, sha},
};

#[test]
fn project_settings_enum_unknown_substitutes_default() {
    run(
        include_bytes!("../../../../tests/parity/project-settings/target-rearseam.3mf"),
        "86bd5b216652980a27bac87f0b234ee449b5b0dd4d8f8e16c987eb040a669dd5",
        "enum-unknown-",
    );
}

#[test]
fn project_settings_enum_aligned_control() {
    run(
        include_bytes!("../../../../tests/parity/project-settings/control-aligned.3mf"),
        "8056c75059017e981d9a15804eb66623d85e5f95a6a07981e151ab85135b1d9b",
        "enum-aligned-",
    );
}

fn run(project: &[u8], expected_sha: &str, prefix: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    assert_eq!(sha(project), expected_sha);
    let case = NativeCase::slice(&runner, project, prefix);
    assert!(case.native.contains("; seam_position = aligned\n"));
    let (api, cli) = case.compare(ares_core::GenerationMetadata::deterministic(
        2026, 10, 20, 0, 0, 0,
    ));
    for bytes in [&api, &cli] {
        assert!(
            std::str::from_utf8(bytes)
                .unwrap()
                .contains("; seam_position = aligned\n")
        );
    }
}
