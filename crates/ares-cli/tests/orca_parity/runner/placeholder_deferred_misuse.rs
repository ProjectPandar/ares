//! `PlaceholderParser.cpp:314-317` deferred values: a deferred placeholder can be printed,
//! but using it in a calculation or a comparison fails (`:862-889`).

use std::fs;

use crate::native_wall_clock::result_comparison_bytes;
use crate::runner::{CaseInputs, ExportedCase, OrcaRunner, application, repo_root};
use crate::{self as parity, presets::VendorProfiles};
use serde_json::json;

const DEFERRED_MESSAGE: &str = "This value is known only after the G-code is generated. It can \
     be printed, but not used in a calculation or a comparison.";

fn export(runner: &OrcaRunner, label: &str, line: &str) -> ExportedCase {
    let profiles = VendorProfiles::load(
        &repo_root().join("OrcaSlicer/resources/profiles"),
        "Creality",
    )
    .unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 V3 SE 0.2 nozzle").unwrap();
    let mut machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    for (key, value) in [
        ("enable_support", "0"),
        ("enable_prime_tower", "0"),
        ("sparse_infill_density", "0%"),
        ("layer_height", "0.14"),
        ("line_width", "0.22"),
        ("outer_wall_line_width", "0.22"),
        ("inner_wall_line_width", "0.22"),
    ] {
        process.insert(key.into(), json!(value));
    }
    let start = format!(
        "{line}\n{}",
        machine["machine_start_gcode"].as_str().unwrap()
    );
    machine.insert("machine_start_gcode".into(), json!(start));
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    parity::normalize_filament_defaults(&mut filaments);
    let exported = runner
        .export_case(
            &CaseInputs {
                label,
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &parity::smoke_overrides(),
            &repo_root().join("tests/parity/cube10.stl"),
        )
        .unwrap();
    application::verify_value(&exported.project, "machine_start_gcode", &json!(start)).unwrap();
    exported
}

fn assert_rejection_matches_native(label: &str, line: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let exported = export(&runner, label, line);
    let artifacts = tempfile::Builder::new()
        .prefix("deferred-placeholder-misuse-")
        .tempdir()
        .unwrap()
        .keep();
    let project = artifacts.join("input.3mf");
    fs::write(&project, &exported.project).unwrap();
    let native = runner.custom_gcode_reference(&project).unwrap();
    assert!(
        String::from_utf8_lossy(&native.stderr).contains(DEFERRED_MESSAGE),
        "Native rejects {line:?} as a deferred value: {:?}",
        String::from_utf8_lossy(&native.stderr)
    );
    let retained = artifacts.join("plate_1.gcode");
    let output = application::cli_command()
        .args(["slice", "-o"])
        .arg(&retained)
        .arg(&project)
        .output()
        .unwrap();
    fs::write(artifacts.join("stderr"), &output.stderr).unwrap();
    assert_eq!(output.status.code(), Some(156), "{artifacts:?}");
    assert_eq!(output.stdout, native.stdout);
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&native.stderr)
    );
    assert_eq!(
        result_comparison_bytes(&fs::read(artifacts.join("result.json")).unwrap()),
        result_comparison_bytes(&native.result_json)
    );
    assert!(!retained.exists());
}

#[test]
fn adding_a_number_to_a_deferred_value_fails_like_native() {
    assert_rejection_matches_native(
        "deferred-placeholder-misuse/add",
        "; TOTAL={print_time_total_sec + 1}",
    );
}

#[test]
fn multiplying_by_a_deferred_value_fails_like_native() {
    assert_rejection_matches_native(
        "deferred-placeholder-misuse/multiply",
        "; SECONDS={2 * print_time_sec}",
    );
}

#[test]
fn comparing_a_deferred_value_fails_like_native() {
    assert_rejection_matches_native(
        "deferred-placeholder-misuse/compare",
        "; LONG={print_time_minute > 5}",
    );
}

#[test]
fn a_variable_holding_a_deferred_value_is_deferred_too() {
    assert_rejection_matches_native(
        "deferred-placeholder-misuse/variable",
        "{local hours = print_time_hour}; HOURS={hours - 1}",
    );
}

#[test]
fn text_holding_a_deferred_value_is_still_concatenated() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let exported = export(
        &runner,
        "deferred-placeholder-misuse/text",
        "; SECONDS={\"s=\" + print_time_sec + 1}",
    );
    let case = runner.slice_case(&exported).unwrap();
    let outcome = parity::compare_case(&case);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}
