//! Print.cpp:1543-1556,1654-1671: a general upper error precedes later lower errors.
use std::fs;

use ares_core::{ConfigOverrides, GenerationMetadata, SliceError, SlicerContext};
use serde_json::json;

use crate::runner::{OrcaRunner, stages};

#[test]
fn print_upper_general_width_precedes_later_minimum_width() {
    verify("3");
}

#[test]
fn print_upper_general_percentage_is_not_in_loading_width_list() {
    verify("600%");
}

fn verify(width: &str) {
    let input =
        include_bytes!("../../../../../tests/parity/validation/minimum-width-ender3s1plus-0.2.3mf");
    let hash = "edb48192a609ede526e8e09e805b845ff99494c0d2d2b31a8655ca9b0ac89110";
    assert_eq!(super::sha256(input), hash);
    let directory = super::evidence_directory("print-upper-general-");
    let path = directory.join("input.3mf");
    fs::write(&path, input).unwrap();
    let override_json = json!({"line_width":width}).to_string();
    let overrides = ConfigOverrides::from_json(override_json.as_bytes()).unwrap();
    fs::write(
        directory.join("invocation.json"),
        serde_json::to_vec_pretty(&json!({
            "input_sha256":hash, "overrides":{"line_width":width},
            "expected_error":"Too large line width",
            "apis":["validate_project_plate","slice_project_plate_validated"]
        }))
        .unwrap(),
    )
    .unwrap();
    let runner = OrcaRunner::from_env().unwrap();
    let oracle = runner.as_ref().map(|runner| {
        let output = directory.join("orca");
        fs::create_dir(&output).unwrap();
        let error = stages::run(
            runner.bin(),
            [
                "--line-width".to_owned(),
                width.to_owned(),
                "--slice".to_owned(),
                "0".to_owned(),
                "--outputdir".to_owned(),
                output.to_str().unwrap().to_owned(),
                path.to_str().unwrap().to_owned(),
            ],
            stages::Stage::Slice,
            &directory,
        )
        .unwrap_err();
        let captured = error.captured.as_ref().unwrap();
        assert_eq!(
            captured.code,
            Some(if cfg!(unix) { 205 } else { -51 }),
            "{error}"
        );
        assert_eq!(captured.stderr, b"Too large line width\n", "{error}");
        assert!(!output.join("plate_1.gcode").exists());
        json!({"code":captured.code,"stderr":"Too large line width\n","evidence":error.evidence})
    });
    let validation = ares_core::validate_project_plate(input, None, &overrides);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let sliced = runtime.block_on(crate::runner::application::slice_project_plate_validated(
        input,
        GenerationMetadata::default(),
        None,
        &overrides,
        SlicerContext::new(0),
    ));
    if let Ok(gcode) = &sliced {
        fs::write(directory.join("ares.gcode"), gcode).unwrap();
    }
    fs::write(
        directory.join("result.json"),
        serde_json::to_vec_pretty(&json!({
            "input_sha256":hash, "oracle":oracle,
            "validation":validation.as_ref().map_err(ToString::to_string),
            "validated_slice":sliced.as_ref().map(Vec::len).map_err(ToString::to_string),
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(
        matches!(validation, Err(SliceError::Validate { message: ref text, .. }) if text == "Too large line width"),
        "Print upper must precede later O4 lower: {validation:?}; {directory:?}"
    );
    assert!(
        matches!(sliced, Err(SliceError::Validate { message: ref text, .. }) if text == "Too large line width"),
        "validated composition must preserve the same first error; {directory:?}"
    );
    assert!(!directory.join("ares.gcode").exists());
}
