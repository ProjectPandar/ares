//! Custom G-code admission parity for unknown function keywords and
//! leading-whitespace error positions
//! (`PlaceholderParser.cpp:1963-1975,2315-2334`, `process_error_message`
//! `:1731-1770`, `resolve_variable` `:989-1002`).
//!
//! Upstream `process_macro` (`:2516-2526`) parses the template value
//! byte-for-byte with no trim and no prefix concatenation; the
//! `ascii_char_skipper_parser` skips leading ' '/'\t'/'\r'/'\n' during
//! parsing only, so they never reach the output, while error positions
//! stay relative to the untrimmed template and skipped blank lines still
//! count toward the reported line number.
//!
//! Two admission fixtures, both exiting through `placeholder_parser_process`
//! (`GCode.cpp:3920-3933`) into `PlaceholderParserError`
//! (`GCode.cpp:3637-3645`, CLI exit 156):
//! - `mk4-unknown-function.3mf`: stock MK4 `layer_change_gcode` plus the
//!   marker `;W4PH_LC_FORMAT={format("%.3f",layer_z)}`. The 2.4.2 grammar
//!   has no `format` keyword, so upstream resolves `format` as a variable
//!   name and throws "Not a variable name". Ares instead parsed any
//!   `identifier(` as a function call and reported
//!   `numeric function argument expected in format(...)` without
//!   line/caret (fixed by the FUNCTION_KEYWORDS grammar gate).
//! - `x1c-blc-leading-blank.3mf`: stock X1C `before_layer_change_gcode` is
//!   empty, so the template value itself starts with the newline before
//!   the same marker (`"\n;W4PH_BLC_FORMAT=..."`); upstream reports
//!   "Parsing error at line 2" because the skipped blank line still
//!   counts. Ares trimmed the template string before parsing, shifting
//!   error offsets onto the trimmed copy ("line 1").
//!
//! Admission standard: Native reference captured twice and byte-stable,
//! both CLIs exit 156, stdout/stderr byte-equal, comparable
//! `result.json`, and no retained G-code.

#![cfg(target_os = "linux")]

use std::{fs, path::Path, process::Command};

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
        "{case}: {artifacts:?}"
    );
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
fn orca_parity_custom_gcode_unknown_function_matches_native() {
    assert_rejection_matches_native("mk4-unknown-function");
}

#[test]
fn orca_parity_custom_gcode_leading_blank_line_matches_native() {
    assert_rejection_matches_native("x1c-blc-leading-blank");
}
