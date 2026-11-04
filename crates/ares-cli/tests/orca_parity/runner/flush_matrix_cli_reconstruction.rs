//! CLI reconstruction (`OrcaSlicer.cpp:3340–3503`) before CONFIG_BLOCK scaling.
use super::{
    CaseInputs, ExportedCase, OrcaRunner, application, repo_root, vg1_two_filament_inputs,
};
use crate::VendorProfiles;
use serde_json::{Value, json};
use std::io::{Cursor, Read, Write};

include!("flush_matrix_cli_reconstruction/helpers.rs");

#[test]
fn flush_matrix_cli_reconstruction_unchanged_nozzle_types_preserve_custom_matrix() {
    check(
        "original",
        &[],
        Some("; flush_volumes_matrix = 0,42,42,0,0,210,210,0\n"),
    );
}

#[test]
fn flush_matrix_cli_reconstruction_missing_matrix_rebuilds_without_nozzle_type_change() {
    check(
        "missing",
        &[("flush_volumes_matrix", None)],
        Some("; flush_volumes_matrix = 0,102,186,0,0,339,621,0\n"),
    );
}

#[test]
fn flush_matrix_cli_reconstruction_rebuild_preserves_multiplier_prefix_and_defaults_second_head() {
    check(
        "single-multiplier",
        &[
            ("flush_volumes_matrix", None),
            ("flush_multiplier", Some(json!(["0.3"]))),
        ],
        Some("; flush_volumes_matrix = 0,102,186,0,0,339,621,0\n"),
    );
}

#[test]
fn flush_matrix_cli_reconstruction_standard_predictor_hit_matches_native() {
    check(
        "standard-predictor-hit",
        &[
            ("flush_volumes_matrix", None),
            ("filament_colour", Some(json!(["#000000", "#545454"]))),
            ("nozzle_flush_dataset", Some(json!(["0", "0", "0", "0"]))),
        ],
        None,
    );
}

#[test]
fn flush_matrix_cli_reconstruction_dual_predictor_datasets_match_native() {
    check(
        "dual-predictors",
        &[
            ("flush_volumes_matrix", None),
            ("filament_colour", Some(json!(["#000000", "#FFFFFF"]))),
            ("nozzle_flush_dataset", Some(json!(["1", "2", "2", "1"]))),
        ],
        None,
    );
}

#[test]
fn flush_matrix_cli_reconstruction_transparent_rgba_and_direction_match_native() {
    check(
        "rgba",
        &[
            ("flush_volumes_matrix", None),
            ("filament_colour", Some(json!(["#F2754E00", "#0000FFFF"]))),
            ("nozzle_flush_dataset", Some(json!(["0", "0", "0", "0"]))),
        ],
        None,
    );
}

#[test]
fn flush_matrix_cli_reconstruction_support_source_and_destination_match_native() {
    check(
        "support",
        &[
            ("flush_volumes_matrix", None),
            ("filament_is_support", Some(json!(["0", "1"]))),
        ],
        None,
    );
}

#[test]
fn flush_matrix_cli_reconstruction_long_retraction_minimum_matches_native() {
    check(
        "long-retraction",
        &[
            ("flush_volumes_matrix", None),
            ("nozzle_flush_dataset", Some(json!(["0", "0", "0", "0"]))),
            ("enable_long_retraction_when_cut", Some(json!("2"))),
            (
                "long_retractions_when_cut",
                Some(json!(["1", "1", "1", "1"])),
            ),
            (
                "filament_long_retractions_when_cut",
                Some(json!(["1", "1"])),
            ),
            (
                "filament_retraction_distances_when_cut",
                Some(json!(["18", "nil"])),
            ),
        ],
        None,
    );
}

#[test]
fn flush_matrix_cli_reconstruction_no_trigger_keeps_malformed_matrix_rejected() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let exported = project(
        &runner,
        "invalid",
        &[(
            "flush_volumes_matrix",
            Some(json!(["0", "140", "140", "0"])),
        )],
    );
    let error = match runner.slice_case(&exported) {
        Ok(_) => panic!("Native must reject a malformed non-rebuilt matrix"),
        Err(error) => error,
    };
    let captured = error.captured.as_ref().unwrap();
    let text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&captured.stdout),
        String::from_utf8_lossy(&captured.stderr)
    );
    const MATRIX_ERROR: &str = "Flush volumes matrix do not match to the correct size!";
    assert!(text.contains(MATRIX_ERROR), "{error}: {text}");
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let ares_error = runtime
        .block_on(crate::runner::application::slice_project(
            &exported.project,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap_err();
    assert_eq!(ares_error.to_string(), MATRIX_ERROR);
    let root = crate::artifacts::root_from_env().unwrap();
    let directory = tempfile::Builder::new()
        .prefix("x2d-fm-invalid-")
        .tempdir_in(root)
        .unwrap()
        .keep();
    std::fs::write(directory.join("input.3mf"), &exported.project).unwrap();
    std::fs::write(directory.join("native.stdout"), &captured.stdout).unwrap();
    std::fs::write(directory.join("native.stderr"), &captured.stderr).unwrap();
    std::fs::write(directory.join("result.json"), serde_json::to_vec_pretty(&json!({
        "label":exported.label, "native_code":captured.code, "native_evidence":error.evidence,
        "native_binary":runner.bin(), "ares_error":ares_error.to_string(), "expected_error":MATRIX_ERROR,
        "no_successful_ares_stream":true,
    })).unwrap()).unwrap();
    eprintln!("nontrigger rejection artifacts {directory:?}");
}

/// O8-X2D-SCALAR closure control: the original UB shape (scalar
/// `nozzle_volume_type` with the four-entry matrix) constructed at
/// runtime from the same export path as the admitted `case-LqoYic`
/// witness. The public ares CLI must fail with the exact upstream
/// matrix-size error (GCode.cpp:5596-5614), exit 1, and produce no
/// G-code. The Native success-on-UB divergence stays recorded in
/// docs/architecture/o8-x2d-scalar-flush-context.md with
/// parity_pass=false; no Native output is committed or asserted here.
#[test]
fn flush_matrix_cli_reconstruction_scalar_four_cell_input_fails_exactly_at_the_cli() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let exported = project(
        &runner,
        "scalar-four-cell",
        &[
            ("nozzle_volume_type", Some(json!(["Standard"]))),
            (
                "flush_volumes_matrix",
                Some(json!(["0", "140", "140", "0"])),
            ),
        ],
    );
    let directory = tempfile::Builder::new()
        .prefix("x2d-fm-scalar-cli-")
        .tempdir()
        .unwrap()
        .keep();
    std::fs::write(directory.join("input.3mf"), &exported.project).unwrap();
    let retained = directory.join("plate_1.gcode");
    let output = crate::runner::application::cli_command()
        .args(["slice", "-o"])
        .arg(&retained)
        .arg(directory.join("input.3mf"))
        .output()
        .unwrap();
    std::fs::write(directory.join("stdout"), &output.stdout).unwrap();
    std::fs::write(directory.join("stderr"), &output.stderr).unwrap();
    std::fs::write(
        directory.join("exit"),
        format!("{:?}\n", output.status.code()),
    )
    .unwrap();
    assert_eq!(output.status.code(), Some(1), "{directory:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "Error: InvalidInput(\"Flush volumes matrix do not match to the correct size!\")\n",
        "{directory:?}"
    );
    assert!(
        !retained.exists(),
        "no exported G-code on the matrix-size error"
    );
}

/// FM closure follow-up (GAP-1): a legal
/// two-value `nozzle_volume_type=[High Flow,High Flow]` input with an
/// absent matrix — the reconstruction runs through the High Flow flush
/// dataset end-to-end, compared against complete Native output (the
/// whole-output `check` compare; the matrix string is pinned from the
/// Native-observed line like the Standard control).
#[test]
fn flush_matrix_cli_reconstruction_high_flow_dataset_matches_native() {
    check(
        "high-flow-rebuild",
        &[
            (
                "nozzle_volume_type",
                Some(json!(["High Flow", "High Flow"])),
            ),
            ("flush_volumes_matrix", None),
        ],
        Some("; flush_volumes_matrix = 0,102,186,0,0,339,621,0\n"),
    );
}

/// The scalar CLI control above must FAIL (not silently pass) when a
/// Native is required but absent (the requires-native meta-test
/// precedent from `cli_custom_gcode_exit`).
#[test]
fn flush_matrix_cli_reconstruction_scalar_controls_require_native() {
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "runner::flush_matrix_cli_reconstruction::flush_matrix_cli_reconstruction_scalar_four_cell_input_fails_exactly_at_the_cli",
            "--nocapture",
        ])
        .env("ARES_REQUIRE_ORCA", "1")
        .env_remove("ARES_ORCA_NATIVE_BIN")
        .output()
        .unwrap();
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line == "running 1 test"),
        "child selector did not run exactly one test: {output:?}"
    );
    let diagnostic = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !output.status.success(),
        "require mode without a native must fail: {diagnostic}"
    );
}
