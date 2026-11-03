//! Custom G-code admission parity for the timelapse overlay split
//! (`GCode.cpp:4667-4674` non-BBL layer-start vs the
//! `insert_timelapse_gcode` lambda `:5162-5167`).
//!
//! Upstream renders `time_lapse_gcode` through two different overlays:
//! the non-BBL layer-start render inside `process_layer` (`:4667-4674`)
//! carries ONLY `layer_num`/`layer_z`/`max_layer_z`, while the
//! `insert_timelapse_gcode` lambda (BBL layer-start and the traditional
//! layer-end insert, `:5162-5167`) additionally maps
//! `most_used_physical_extruder_id`/`curr_physical_extruder_id` and the
//! `timelapse_pos_x`/`timelapse_pos_y`/`has_timelapse_safe_pos` park
//! position. A non-BBL machine referencing
//! `{most_used_physical_extruder_id}` in `time_lapse_gcode` therefore
//! fails symbol resolution — "Not a variable name" with the identifier's
//! line/caret (`resolve_variable` `:988-1000`, `process_error_message`
//! `:1731-1770`) — and exits through `PlaceholderParserError` (CLI 156).
//!
//! Fixture: stock MK4 `cube_with_hole` (`time_lapse_gcode` empty in
//! stock) appending `;W4PH_TLMPE={most_used_physical_extruder_id}`.
//! RED (#169 discovery row `tlmpe/mk4`): `timelapse.rs::append` injected
//! the lambda-only physical extruder ids unconditionally, so Ares
//! accepted the template (exit 0, rendered `0`) while Native rejects on
//! both official and seeded replay natives.
//!
//! Admission standard: Native reference captured twice and byte-stable,
//! both CLIs exit 156, stdout/stderr byte-equal, comparable
//! `result.json`, and no retained G-code.

#![cfg(target_os = "linux")]

use std::{fs, path::Path};

use crate::native_wall_clock::result_comparison_bytes;

#[test]
fn custom_gcode_timelapse_overlay_mk4_matches_native() {
    let case = "mk4-tlmpe";
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
    let diagnostic = b"Not a variable name";
    assert!(
        native
            .stderr
            .windows(diagnostic.len())
            .any(|w| w == diagnostic),
        "upstream rejects the lambda-only variable on the non-BBL path: {:?}",
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
