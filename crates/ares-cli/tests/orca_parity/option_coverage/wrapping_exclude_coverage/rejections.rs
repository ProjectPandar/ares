//! Print.cpp:932-979,1304-1310: by-layer wrapping polygon admission.
//! PartPlate.cpp:5454-5485 / OrcaSlicer.cpp:5717-5724: arrange can fail earlier.
use super::*;
use ares_core::{ConfigOverrides, GenerationMetadata, SliceError, SlicerContext};
use runner::stages::{FailureKind, Stage};

#[test]
fn original_orca_and_ares_reject_wrapping_polygon_intersection() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = parity::artifacts::root_from_env().unwrap();
    let points = json!(["0x0", "220x0", "220x220", "0x220"]);
    let baseline =
        input::export(&runner, &points, false, "bounded-wrapping/overlap-disabled").unwrap();
    let bounds = footprint(&baseline.project);
    assert!(bounds[0] > 0.0 && bounds[1] > 0.0 && bounds[2] < 220.0 && bounds[3] < 220.0);
    let control = runner.slice_case(&baseline).unwrap();
    assert!(
        !std::str::from_utf8(&control.reference)
            .unwrap()
            .lines()
            .any(|line| line.starts_with(";W6_WRAP "))
    );
    let outcome = parity::compare_case(&control);
    assert_eq!(outcome.status, "PASS", "{}; {root:?}", outcome.detail);
    let enabled = runner::wrapping_input::enable(baseline, &root);
    assert_eq!(footprint(&enabled.project), bounds);
    let native = rejection(&runner, &enabled, -64);
    let diagnostic = native["stderr"].as_str().unwrap().trim_end();
    assert!(
        diagnostic.contains("too close to clumping detection area"),
        "{native}"
    );
    let validation =
        ares_core::validate_project_plate(&enabled.project, None, &ConfigOverrides::default());
    let sliced = validated(&enabled.project);
    if let Ok(output) = &sliced {
        parity::artifacts::overwrite(&root.join("wrapping-intersection-ares.gcode"), output)
            .unwrap();
    }
    parity::artifacts::overwrite(&root.join("wrapping-exclude-area-rejection.json"), &serde_json::to_vec_pretty(&json!({
        "requested":points,"footprint":bounds,"native":native,
        "public_validation":format!("{validation:?}"),
        "validated_slice":sliced.as_ref().map(|output| output.len()).map_err(ToString::to_string),
    })).unwrap()).unwrap();
    #[cfg(target_os = "linux")]
    cli_rejection(&root, &native);
    assert!(
        matches!(validation, Err(ref error) if error.to_string().trim_end() == diagnostic),
        "Ares public validation disagrees with Native: {validation:?}; {root:?}"
    );
    assert!(
        matches!(sliced, Err(ref error) if error.to_string().trim_end() == diagnostic),
        "Ares validated slice disagrees with Native; {root:?}"
    );
}

#[test]
fn original_orca_and_ares_reject_arranged_wrapping_object_outside_bed() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = parity::artifacts::root_from_env().unwrap();
    let points = json!(["0x0", "220x0", "220x220", "0x220"]);
    let enabled =
        input::export(&runner, &points, true, "bounded-wrapping/overlap-enabled").unwrap();
    let bounds = footprint(&enabled.project);
    assert!(
        bounds[0] < 0.0 || bounds[1] < 0.0 || bounds[2] > 220.0 || bounds[3] > 220.0,
        "arrange rejection witness requires relocated footprint: {bounds:?}"
    );
    let native = rejection(&runner, &enabled, -50);
    let sliced = validated(&enabled.project);
    parity::artifacts::overwrite(&root.join("wrapping-arrange-rejection.json"), &serde_json::to_vec_pretty(&json!({
        "requested":points,"relocated_footprint":bounds,"native":native,
        "validated_slice":sliced.as_ref().map(|output| output.len()).map_err(ToString::to_string),
    })).unwrap()).unwrap();
    let diagnostic = native["result"]["error_string"].as_str().unwrap();
    assert!(
        matches!(sliced, Err(SliceError::NoSuitableObjects(ref message)) if message == diagnostic),
        "Ares validated slice disagrees with Native arrange rejection; {root:?}"
    );
}

fn validated(project: &[u8]) -> Result<Vec<u8>, SliceError> {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project_plate_validated(
            project,
            GenerationMetadata::default(),
            None,
            &ConfigOverrides::default(),
            SlicerContext::new(0),
        ))
}

fn rejection(runner: &OrcaRunner, exported: &runner::ExportedCase, code: i32) -> Value {
    let error = runner
        .slice_case(exported)
        .err()
        .expect("Native must reject wrapping input");
    assert_eq!(error.stage, Stage::Slice, "{error}");
    assert_eq!(error.kind, FailureKind::Process, "{error}");
    let captured = error.captured.as_ref().unwrap();
    assert_eq!(
        captured.code,
        Some(if cfg!(unix) {
            code.rem_euclid(256)
        } else {
            code
        }),
        "{error}"
    );
    let command: Value = serde_json::from_slice(
        &std::fs::read(error.evidence.as_ref().unwrap().join("command.json")).unwrap(),
    )
    .unwrap();
    let args = command["args"].as_array().unwrap();
    let output = args
        .windows(2)
        .find(|pair| pair[0].as_str() == Some("--outputdir"))
        .unwrap()[1]
        .as_str()
        .unwrap();
    let result_bytes = std::fs::read(std::path::Path::new(output).join("result.json")).unwrap();
    let result: Value = serde_json::from_slice(&result_bytes).unwrap();
    assert_eq!(result["return_code"], json!(code));
    json!({"code":captured.code,"result":result,"result_text":String::from_utf8(result_bytes).unwrap(),"evidence":error.evidence,
        "stdout":String::from_utf8_lossy(&captured.stdout),"stderr":String::from_utf8_lossy(&captured.stderr)})
}

#[cfg(target_os = "linux")]
fn cli_rejection(root: &std::path::Path, native: &Value) {
    let directory = root.join("wrapping-cli");
    std::fs::create_dir_all(&directory).unwrap();
    let gcode = directory.join("plate_1.gcode");
    let output = crate::runner::application::cli_command()
        .args(["slice", "-o"])
        .arg(&gcode)
        .arg(root.join("wrapping-fixed-placement/wrapping-input.3mf"))
        .output()
        .unwrap();
    for (name, bytes) in [("stdout", &output.stdout), ("stderr", &output.stderr)] {
        std::fs::write(directory.join(name), bytes).unwrap();
    }
    std::fs::write(
        directory.join("exit"),
        format!("{:?}\n", output.status.code()),
    )
    .unwrap();
    assert_eq!(
        output.status.code(),
        Some(192),
        "CLI disagrees with Native; {directory:?}"
    );
    assert_eq!(output.stdout, native["stdout"].as_str().unwrap().as_bytes());
    assert_eq!(output.stderr, native["stderr"].as_str().unwrap().as_bytes());
    assert_eq!(
        crate::native_wall_clock::result_comparison_bytes(
            &std::fs::read(directory.join("result.json")).unwrap()
        ),
        crate::native_wall_clock::result_comparison_bytes(
            native["result_text"].as_str().unwrap().as_bytes()
        )
    );
    assert!(!gcode.exists(), "collision rejected before emission");
}
