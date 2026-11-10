//! Custom G-code admission parity for bare vector variables in scalar
//! context (`PlaceholderParser.cpp:1086-1211` `vector_element_to_expr`,
//! default throw `:1204`; `variable_value` dispatch `:1348-1356`).
//!
//! Upstream resolves a brace variable through `variable_value`: vector
//! options enter `vector_element_to_expr`, where a bare (no `[index]`)
//! reference only auto-serializes float vectors — `case coFloats` /
//! `coFloatsOrPercents` pick `size() == 1 ? get_at(0) :
//! get_at(get_extruder_id())` — and every other vector type (coInts,
//! coStrings, coBools, …) throws
//! "Referencing a vector variable when scalar is expected" with the
//! identifier's line/caret (`process_error_message` `:1731-1770`).
//!
//! Two stock fixtures (MK4 and K1) whose `machine_start_gcode` appends
//! `;ARES_PROBE_MS_VARS N={nozzle_temperature} B={bed_temperature}
//! F={filament_type[0]} T={total_layer_count}`. `nozzle_temperature` and
//! `bed_temperature` are `coInts` (`PrintConfig.cpp:6644`, `:11315`), so
//! Native rejects on the bare `nozzle_temperature` reference (line 83 MK4 /
//! line 9 K1, caret under the identifier). RED: Ares resolved bare vectors
//! through the `Value::scalar()` first-element fallback, accepted the
//! template, and rendered `N=220 B=35|60 F=PLA T=50`.
//!
//! The indexed `filament_type[0]` and scalar `total_layer_count` in the
//! same line witness that indexed `vector[i]` and scalar references keep
//! working; the legacy `[var]` spelling keeps upstream auto-serialization
//! (covered by the stock templates of the same fixtures).
//!
//! Admission standard: Native reference captured twice and byte-stable,
//! both CLIs exit 156, stdout/stderr byte-equal, comparable `result.json`,
//! and no retained G-code.

#![cfg(target_os = "linux")]

use std::{fs, path::Path};

use crate::native_wall_clock::result_comparison_bytes;

fn assert_rejection_matches_native(case: &str) {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/parity/validation/cli-exit/custom-gcode")
        .join(format!("{case}.3mf"));
    let artifacts = tempfile::Builder::new()
        .prefix(&format!("cli-custom-gcode-{case}-"))
        .tempdir()
        .unwrap()
        .keep();
    let Some(runner) = crate::runner::OrcaRunner::from_env().unwrap() else {
        return;
    };
    // Attribution gate: the Native failure reference must be byte-stable
    // before the admission comparison is meaningful.
    let native = runner.custom_gcode_reference(&fixture).unwrap();
    let rerun = runner.custom_gcode_reference(&fixture).unwrap();
    assert_eq!(native.directory, rerun.directory);
    assert_eq!(native.stdout, rerun.stdout);
    assert_eq!(native.stderr, rerun.stderr);
    assert_eq!(native.result_json, rerun.result_json);
    fs::write(artifacts.join("native.stdout"), &native.stdout).unwrap();
    fs::write(artifacts.join("native.stderr"), &native.stderr).unwrap();
    fs::write(artifacts.join("native.result.json"), &native.result_json).unwrap();
    fs::copy(
        native.directory.join("receipt.json"),
        artifacts.join("native.receipt.json"),
    )
    .unwrap();
    assert!(
        native
            .stderr
            .starts_with(b"Failed to generate G-code for invalid custom G-code."),
        "fixture must keep the custom-G-code rejection shape"
    );
    let diagnostic = b"Referencing a vector variable when scalar is expected";
    assert!(
        native
            .stderr
            .windows(diagnostic.len())
            .any(|w| w == diagnostic),
        "upstream rejects the bare vector reference: {:?}",
        String::from_utf8_lossy(&native.stderr)
    );

    let retained = artifacts.join("plate_1.gcode");
    let output = crate::runner::application::cli_command()
        .args(["slice", "-o"])
        .arg(&retained)
        .arg(&fixture)
        .output()
        .unwrap();
    fs::write(artifacts.join("stdout"), &output.stdout).unwrap();
    fs::write(artifacts.join("stderr"), &output.stderr).unwrap();
    fs::write(
        artifacts.join("exit"),
        format!("{:?}\n", output.status.code()),
    )
    .unwrap();
    assert_eq!(output.status.code(), Some(156), "{case}: {artifacts:?}");
    assert_eq!(output.stdout, native.stdout);
    assert_eq!(
        output.stderr, native.stderr,
        "{case}: rejection diagnostics must match upstream byte for byte"
    );
    assert_eq!(
        result_comparison_bytes(&fs::read(artifacts.join("result.json")).unwrap()),
        result_comparison_bytes(&native.result_json),
        "complete native JSON: generic error, applied metadata"
    );
    assert!(
        !retained.exists(),
        "no final exported G-code on custom-template failure"
    );
}

#[test]
fn custom_gcode_vector_scalar_mk4_matches_native() {
    assert_rejection_matches_native("mk4-ms-vector-scalar");
}

#[test]
fn custom_gcode_vector_scalar_k1_matches_native() {
    assert_rejection_matches_native("k1-ms-vector-scalar");
}
