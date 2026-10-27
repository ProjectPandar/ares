//! Config.cpp:657-676 / Config.hpp:2032-2040: EnableSilent enum substitution.
//! Config.cpp:947-1000 / Config.cpp:832-867 / Config.hpp:784-790: array-valued
//! scalar options join with ',' and take the first numeric token per type.
use crate::{
    OrcaRunner,
    runner::native_api_cli::{NativeCase, sha},
    runner::native_stream_case,
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

fn run_array_scalar_case(name: &str, project: &[u8], expected_sha: &str) {
    native_stream_case::check(
        (
            "project-settings-array-scalar",
            &[
                "Config.cpp:947-1000",
                "Config.cpp:832-867",
                "Config.hpp:784-790",
            ],
        ),
        name,
        project,
        expected_sha,
        (false, |reference: &NativeCase, _: &NativeCase| {
            assert!(
                reference
                    .native
                    .contains("; support_interface_speed = 42\n")
            );
        }),
    );
}

#[test]
fn project_settings_array_scalar_iface_speed_arr42() {
    run_array_scalar_case(
        "mk4-iface-speed-arr42",
        include_bytes!("../../../../tests/parity/project-settings/mk4-iface-speed-arr42.3mf"),
        "8b392435614ece69fc0dfb654dcd25e193b69757395782dd4fe9a3b737de5053",
    );
}

#[test]
fn project_settings_array_scalar_iface_speed_arr42_77() {
    run_array_scalar_case(
        "mk4-iface-speed-arr42-77",
        include_bytes!("../../../../tests/parity/project-settings/mk4-iface-speed-arr42-77.3mf"),
        "c7ed3e9d84c4a3c08349df5dc3ed5cb344eb36c26792e0d0db53c5f49d2f263e",
    );
}
