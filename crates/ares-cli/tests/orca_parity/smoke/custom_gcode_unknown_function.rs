//! Custom G-code admission parity for an unknown function keyword
//! (`PlaceholderParser.cpp:2315-2334`, `resolve_variable` `:989-1002`).
//!
//! RED fixture: stock MK4 `cube_with_hole` export whose
//! `layer_change_gcode` keeps the stock template and appends the marker
//! `;W4PH_LC_FORMAT={format("%.3f",layer_z)}` (line 3). The 2.4.2 grammar
//! has no `format` function keyword, so upstream parses `format` as a
//! variable reference, `resolve_variable` throws "Not a variable name"
//! with the source line and caret, and the export aborts through
//! `placeholder_parser_process` (`GCode.cpp:3920-3933`) into
//! `PlaceholderParserError` (`GCode.cpp:3637-3645`, CLI exit 156).
//! Ares's expression parser instead accepted any `identifier(` as a
//! function call and failed argument typing with
//! `numeric function argument expected in format(...)` — no line/caret —
//! so the CLI diagnostics diverged byte-wise on the same rejection.
//!
//! Admission standard: Native reference captured twice and byte-stable,
//! both CLIs exit 156, stdout/stderr byte-equal, comparable
//! `result.json`, and no retained G-code.

#![cfg(target_os = "linux")]

use std::{fs, path::Path, process::Command};

use crate::native_wall_clock::result_comparison_bytes;

#[test]
fn orca_parity_custom_gcode_unknown_function_matches_native() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/parity/validation/cli-exit/custom-gcode/mk4-unknown-function.3mf");
    let artifacts = tempfile::Builder::new()
        .prefix("cli-custom-gcode-unknown-function-")
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
    assert!(
        native
            .stderr
            .windows(b"Not a variable name".len())
            .any(|window| window == b"Not a variable name"),
        "upstream rejects the unknown function keyword as a variable name: {:?}",
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
    assert_eq!(
        output.status.code(),
        Some(156),
        "{case}: {artifacts:?}",
        case = "mk4-unknown-function"
    );
    assert_eq!(output.stdout, native.stdout);
    assert_eq!(
        output.stderr, native.stderr,
        "unknown function keyword must reject with upstream's variable-name diagnostic"
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
