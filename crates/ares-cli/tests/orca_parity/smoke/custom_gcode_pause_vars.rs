//! Custom G-code admission parity for per-layer pause/template renders
//! (`GCode.cpp:4174-4258`, the custom G-code per print-z table stored in
//! `Metadata/custom_gcode_per_layer.xml`, `bbs_3mf.cpp:3208-3292`).
//!
//! `machine_pause_gcode` (`:4240`) and `template_custom_gcode`
//! (`:4246`) render through `placeholder_parser_process(name, template,
//! current_extruder_id)` with NO DynamicConfig overlay. Layer variables
//! instead belong to explicit overlays, including wipe-tower/toolchange
//! (`GCode.cpp:783-784`, `:853-854`, `:1359-1360`, `:7728-7730`,
//! `:7771-7773`, `:7867-7869`, `:8019-8021`), startup filament (:3149),
//! print end (:3476-3479), layer hooks (:4636-4685), timelapse/wrapping
//! (`:5159-5161`, `:5219-5221`), and role change (:6755-6756).
//! The base `PlaceholderParser`
//! carries no `layer_num`, so a pause template referencing `{layer_num}`
//! fails symbol resolution — "Not a variable name" with the identifier's
//! line/caret (`resolve_variable` `:988-1000`, `process_error_message`
//! `:1731-1770`) — and exits through `PlaceholderParserError` (CLI 156).
//!
//! Fixture: stock MK4 two-filament tower plus a
//! `Metadata/custom_gcode_per_layer.xml` with a `PausePrint` entry at
//! `top_z 0.4` and a `Custom` entry at `top_z 2.0`, and
//! `machine_pause_gcode = ;PROBE_PAUSE n={layer_num} z={layer_z}`.
//! RED: `placeholders::base_config` seeded `layer_num`/`layer_z` into
//! EVERY render context (`insert_runtime_placeholders`), so Ares
//! accepted the template (exit 0, rendered stale `n=0 z=0`) while
//! Native rejects on both official and seeded replay natives.
//!
//! Admission standard: Native reference captured twice and byte-stable,
//! both CLIs exit 156, stdout/stderr byte-equal, comparable
//! `result.json`, and no retained G-code.

#![cfg(target_os = "linux")]

use std::{fs, path::Path, process::Command};

use crate::native_wall_clock::result_comparison_bytes;

#[test]
fn custom_gcode_pause_vars_mk4_matches_native() {
    let case = "mk4-pause-vars";
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
        "upstream rejects the overlay-only variables on the pause render: {:?}",
        String::from_utf8_lossy(&native.stderr)
    );

    let retained = artifacts.join("plate_1.gcode");
    let output = Command::new(env!("CARGO_BIN_EXE_ares"))
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
