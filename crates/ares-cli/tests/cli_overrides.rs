//! E2E rejection coverage for `ares slice --overrides` (M854).
//!
//! `ConfigOverrides::from_json` is the public boundary validation of the
//! upstream `change_opt_value` → `ConfigBase::set_deserialize` flow
//! (`Config.cpp:573-620`). Out-of-range values surface through the CLI
//! setup validation (`OrcaSlicer.cpp:7197` `m_config.validate(true)` =
//! `Slic3r::validate`, `PrintConfig.cpp:10337+`): special-case keys report
//! `invalid value` (`PrintConfig.cpp:10339-10346`) and the O3 loop reports
//! `[min,max]` (`PrintConfig.cpp:10561`); the shared `std::map` keeps the
//! first message per key (`emplace`, `PrintConfig.cpp:10559`).

use std::{fs, path::PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn slice_with_overrides(overrides: &str, output: &std::path::Path) -> assert_cmd::assert::Assert {
    Command::cargo_bin("ares")
        .unwrap()
        .args([
            "slice",
            "-o",
            output.to_str().unwrap(),
            repo_root()
                .join("tests/parity/orca_cli_ender3.3mf")
                .to_str()
                .unwrap(),
            "--overrides",
            overrides,
        ])
        .assert()
}

#[test]
fn overrides_reject_unknown_option_key() {
    let temp = tempfile::tempdir().unwrap();
    slice_with_overrides(
        r#"{"not_an_orca_option":"1"}"#,
        &temp.path().join("out.gcode"),
    )
    .failure()
    .stderr(
        predicate::str::contains("not_an_orca_option")
            .and(predicate::str::contains("unknown Orca option")),
    );
}

#[test]
fn overrides_reject_malformed_value_token() {
    let temp = tempfile::tempdir().unwrap();
    slice_with_overrides(
        r#"{"layer_height":"not-a-float"}"#,
        &temp.path().join("out.gcode"),
    )
    .failure()
    .stderr(predicate::str::contains("layer_height"));
}

#[test]
fn overrides_reject_out_of_range_value_with_upstream_bounds() {
    let temp = tempfile::tempdir().unwrap();
    // The O3 `is_value_valid` loop (`PrintConfig.cpp:10521-10559`) reports
    // the `[min,max]` bounds with 6 fraction digits. Native CLI
    // `--filament-max-volumetric-speed -1` emits this exact line under
    // `Params in command line error:` (the unset max renders as the
    // float-max sentinel `PrintConfig.cpp:10559` serializes).
    slice_with_overrides(
        r#"{"filament_max_volumetric_speed":"-1"}"#,
        &temp.path().join("out.gcode"),
    )
    .failure()
    .stderr(
        predicate::str::contains("Params in command line error").and(predicate::str::contains(
            "filament_max_volumetric_speed: -1 not in range \
                 [0.000000,340282346638528859811704183484516925440.000000]",
        )),
    );
}

#[test]
fn overrides_reject_special_case_key_with_first_native_message() {
    let temp = tempfile::tempdir().unwrap();
    // `layer_height` hits the special-case checks at the top of
    // `Slic3r::validate` (`PrintConfig.cpp:10339-10346`) before the range
    // loop, and the shared error map keeps the first message per key
    // (`emplace`, `PrintConfig.cpp:10559`): native CLI `--layer-height -5`
    // reports `invalid value -5.000000`, not `not in range`.
    slice_with_overrides(r#"{"layer_height":"-5"}"#, &temp.path().join("out.gcode"))
        .failure()
        .stderr(
            predicate::str::contains("Params in command line error").and(predicate::str::contains(
                "layer_height: invalid value -5.000000",
            )),
        );
}

#[test]
fn overrides_apply_vector_comma_tokens() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("out.gcode");
    slice_with_overrides(
        r#"{"layer_height":"0.3","nozzle_temperature":"210,220"}"#,
        &output,
    )
    .success();
    let gcode = fs::read_to_string(&output).unwrap();
    assert!(gcode.contains("; layer_height = 0.3"));
    assert!(gcode.contains("; nozzle_temperature = 210,220"));
}
