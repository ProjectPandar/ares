//! E2E rejection coverage for `ares slice --overrides` (M854).
//!
//! `ConfigOverrides::from_json` is the public boundary validation of the
//! upstream `change_opt_value` → `ConfigBase::set_deserialize` flow
//! (`Config.cpp:573-620`). Each rejection class maps to an upstream error
//! surface: unknown key → `UnknownOptionException`, malformed token →
//! `BadOptionValueException`, out-of-range → `is_value_valid`
//! (`Config.cpp:321-337`) reporting `[min,max]` (`PrintConfig.cpp:10561`).

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
    // `layer_height` min is 0; a negative value fails
    // `ConfigOptionDef::is_value_valid` and reports the upstream
    // `[min,max]` bounds in the diagnostic.
    slice_with_overrides(r#"{"layer_height":"-5"}"#, &temp.path().join("out.gcode"))
        .failure()
        .stderr(
            predicate::str::contains("layer_height")
                .and(predicate::str::contains("not in range [")),
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
