//! Populated selection and later volume rejection must not migrate channels.
use crate::runner::{OrcaRunner, stages};
use ares_core::{ConfigOverrides, GenerationMetadata, SliceError, SlicerContext};
use serde_json::json;
use std::fs;

const OUTSIDE: &str = "One of the plate is empty or has no object fully inside it. Please check that the 3mf contains no empty plate in Orca Slicer before uploading.";

#[test]
fn empty_selected_plate_control_populated_plate_two_preserves_native_stream() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let input = include_bytes!("../../../../tests/parity/validation/print-bed-two-plates-min.3mf");
    let root = crate::artifacts::root_from_env().unwrap();
    let directory = tempfile::Builder::new()
        .prefix("empty-plate-populated-")
        .tempdir_in(root)
        .unwrap()
        .keep();
    let path = directory.join("input.3mf");
    let output = directory.join("native");
    fs::write(&path, input).unwrap();
    fs::create_dir(&output).unwrap();
    stages::run(
        runner.bin(),
        [
            "--slice",
            "2",
            "--outputdir",
            output.to_str().unwrap(),
            path.to_str().unwrap(),
        ]
        .into_iter()
        .map(str::to_owned),
        stages::Stage::Slice,
        &directory,
    )
    .unwrap();
    let native = fs::read(output.join("plate_2.gcode")).unwrap();
    let actual = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_project_plate(
            input.as_slice(),
            GenerationMetadata::default(),
            Some(2),
            &ConfigOverrides::default(),
            SlicerContext::new(0),
        ))
        .unwrap();
    fs::write(directory.join("ares.gcode"), &actual).unwrap();
    fs::write(directory.join("orca.gcode"), &native).unwrap();
    let compared = crate::golden::compare_ordered_bytes_generator_only(&native, &actual);
    fs::write(directory.join("result.json"), serde_json::to_vec_pretty(&json!({
        "label": "empty-plate-control/plate2", "input": path,
        "status": if compared.is_ok() { "PASS" } else { "DIVERGENT" }, "difference": compared.as_ref().err(),
    })).unwrap()).unwrap();
    compared.unwrap();
}

#[test]
fn empty_selected_plate_control_nonempty_outside_volume_keeps_later_error() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let input =
        include_bytes!("../../../../tests/parity/validation/print-volume-kobra3-off-bed.3mf");
    let root = crate::artifacts::root_from_env().unwrap();
    let directory = tempfile::Builder::new()
        .prefix("empty-plate-outside-")
        .tempdir_in(root)
        .unwrap()
        .keep();
    let path = directory.join("input.3mf");
    let output = directory.join("native");
    fs::write(&path, input).unwrap();
    fs::create_dir(&output).unwrap();
    // Import all plates as in the original off-bed O6 control; explicit plate1
    // would instead hit this fixture's importer membership/read boundary.
    let error = stages::run(
        runner.bin(),
        [
            "--slice",
            "0",
            "--outputdir",
            output.to_str().unwrap(),
            path.to_str().unwrap(),
        ]
        .into_iter()
        .map(str::to_owned),
        stages::Stage::Slice,
        &directory,
    )
    .unwrap_err();
    let captured = error.captured.as_ref().unwrap();
    assert_eq!(captured.code, Some(if cfg!(unix) { 206 } else { -50 }));
    assert!(captured.stderr.is_empty());
    let native: serde_json::Value =
        serde_json::from_slice(&fs::read(output.join("result.json")).unwrap()).unwrap();
    assert_eq!(native["return_code"], -50);
    assert_eq!(native["error_string"], OUTSIDE);
    let actual = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_project_plate(
            input.as_slice(),
            GenerationMetadata::default(),
            None,
            &ConfigOverrides::default(),
            SlicerContext::new(0),
        ));
    fs::write(directory.join("result.json"), serde_json::to_vec_pretty(&json!({
        "label": "empty-plate-control/outside", "native": native, "ares": actual.as_ref().err().map(ToString::to_string),
        "ares_debug": format!("{actual:?}"), "source": "OrcaSlicer.cpp:5710-5724; unchanged print volume gate",
    })).unwrap()).unwrap();
    assert!(matches!(actual, Err(SliceError::NoSuitableObjects(ref text)) if text == OUTSIDE));
}
