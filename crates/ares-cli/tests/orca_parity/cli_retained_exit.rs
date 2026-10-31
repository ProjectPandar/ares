//! GCode.cpp:2030,2166-2192 -> OrcaSlicer.cpp:6209-6227: export BEFORE -102.
#![cfg(target_os = "linux")]

use std::{fs, path::Path, process::Command};

use crate::{
    cli_retained_input::{normalize_generator, second_plate},
    native_wall_clock::result_comparison_bytes,
};

fn fixtures() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/parity/validation")
}

type NativeChannels = (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>);

fn native_reference(input: &Path, second: bool, artifacts: &Path) -> Option<NativeChannels> {
    let runner = crate::runner::OrcaRunner::from_env().unwrap()?;
    let native = runner.retained_cli_reference(input, second).unwrap();
    let hit = runner.retained_cli_reference(input, second).unwrap();
    assert_eq!(native.directory, hit.directory);
    assert_eq!(native.stdout, hit.stdout);
    assert_eq!(native.stderr, hit.stderr);
    assert_eq!(native.result_json, hit.result_json);
    assert_eq!(native.gcode, hit.gcode);
    fs::write(artifacts.join("native.stdout"), &native.stdout).unwrap();
    fs::write(artifacts.join("native.stderr"), &native.stderr).unwrap();
    fs::write(artifacts.join("native.result.json"), &native.result_json).unwrap();
    fs::copy(
        native.directory.join("receipt.json"),
        artifacts.join("native.receipt.json"),
    )
    .unwrap();
    let gcode = native.gcode.unwrap();
    assert!(
        !gcode.is_empty(),
        "Native must retain complete nonempty export"
    );
    fs::write(artifacts.join("native.gcode"), &gcode).unwrap();
    Some((native.stdout, native.stderr, native.result_json, gcode))
}

fn run_cli(input: &Path, second: bool, artifacts: &Path) {
    let Some((stdout, stderr, result_json, native)) = native_reference(input, second, artifacts)
    else {
        return;
    };
    let retained = artifacts.join(if second {
        "plate_2.gcode"
    } else {
        "plate_1.gcode"
    });
    let mut command = Command::new(env!("CARGO_BIN_EXE_ares"));
    command.arg("slice");
    if second {
        command.args(["--plate", "2"]);
    }
    let output = command
        .arg("-o")
        .arg(&retained)
        .arg(input)
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
        Some(154),
        "typed -102 exit: {artifacts:?}"
    );
    assert_eq!(output.stdout, stdout);
    assert_eq!(output.stderr, stderr);
    assert_eq!(
        result_comparison_bytes(&fs::read(artifacts.join("result.json")).unwrap()),
        result_comparison_bytes(&result_json),
        "complete native result.json, including applied configuration: {artifacts:?}"
    );
    let actual = fs::read_to_string(&retained).unwrap();
    let native = String::from_utf8(native).unwrap();
    assert!(!actual.is_empty());
    assert_eq!(
        normalize_generator(&actual, "Ares"),
        normalize_generator(&native, "OrcaSlicer"),
        "retained complete stream: generator-only normalization: {artifacts:?}"
    );
}

#[test]
fn retained_reference_requires_native_not_frozen_outputs() {
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "cli_retained_exit::cli_unprintable_area_retains_complete_native_gcode_before_exit_154",
            "--nocapture",
        ])
        .env("ARES_REQUIRE_ORCA", "1")
        .env_remove("ARES_ORCA_NATIVE_BIN")
        .output()
        .unwrap();
    let diagnostic = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        diagnostic.contains("running 1 test"),
        "selector drift: {diagnostic}"
    );
    assert!(
        !output.status.success() && diagnostic.contains("ARES_ORCA_NATIVE_BIN"),
        "frozen retained reference hid required Native absence: {diagnostic}"
    );
}

#[test]
fn cli_unprintable_area_retains_complete_native_gcode_before_exit_154() {
    let temp = tempfile::Builder::new()
        .prefix("cli-retained-first-")
        .tempdir()
        .unwrap()
        .keep();
    run_cli(
        &fixtures().join("print-volume-stock-circle-inward.3mf"),
        false,
        &temp,
    );
}

#[test]
fn cli_unprintable_area_reports_selected_nonminimum_plate() {
    let temp = tempfile::Builder::new()
        .prefix("cli-retained-selected-plate-")
        .tempdir()
        .unwrap()
        .keep();
    let input = second_plate(&temp, &fixtures(), "-1.3");
    run_cli(&input, true, &temp);
}

/// Model.cpp:3423-3469 / BuildVolume.cpp:393-396 accept all actual vertices;
/// GCode.cpp:2166-2173 then rejects the complete retained export with -102.
#[tokio::test]
async fn circle_bed_per_vertex_accepts_inward_cylinder_then_rejects_export() {
    use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext, slice_project_plate};

    let artifacts = tempfile::Builder::new()
        .prefix("core-o6-retained-")
        .tempdir()
        .unwrap()
        .keep();
    let input = fixtures().join("print-volume-stock-circle-inward.3mf");
    let Some((_, _, _, expected)) = native_reference(&input, false, &artifacts) else {
        return;
    };
    let expected = expected.as_slice();
    let result = slice_project_plate(
        fs::read(&input).unwrap(),
        GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
        None,
        &ConfigOverrides::default(),
        SlicerContext::new(0),
    )
    .await;
    assert!(
        result.is_err(),
        "Native rejects at export after Model acceptance"
    );
    let error = result.unwrap_err();
    assert_eq!(
        error.to_string(),
        "Found G-code in unprintable area of multi-extruder printers after slicing. Please make sure the 3mf file can be successfully sliced in the latest Orca Slicer."
    );
    let ares_core::SliceError::ToolpathOutsidePrintableArea {
        error_code, gcode, ..
    } = error
    else {
        panic!("Native rejects in the processor export gate, not Model eligibility");
    };
    assert_eq!(error_code, 4);
    fs::write(artifacts.join("ares.gcode"), &gcode).unwrap();
    let normalized = |bytes: &[u8]| {
        bytes
            .split_inclusive(|byte| *byte == b'\n')
            .filter(|line| !line.starts_with(b"; generated by"))
            .flat_map(|line| line.iter().copied())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        normalized(&gcode),
        normalized(expected),
        "rejected output must retain Native whole bytes; only generator line may differ"
    );
}
