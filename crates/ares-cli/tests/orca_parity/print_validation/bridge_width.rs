//! Print.cpp:1675-1692: source-owned bridge-width region/role bounds.
use std::fs;

use ares_core::{ConfigOverrides, GenerationMetadata, SliceError, SlicerContext};
use serde_json::{Value, json};

use crate::runner::{OrcaRunner, stages};

#[test]
fn print_bridge_width_original_thin_precedes_layer_reset_check() {
    verify(
        super::RELATIVE_INPUT,
        super::RELATIVE_SHA256,
        json!({"bridge_line_width":"0.1"}),
        "Too small line width",
    );
}

#[test]
fn print_bridge_width_scoped_upper_precedes_layer_reset_check() {
    let input =
        include_bytes!("../../../../../tests/parity/validation/print-bridge-region-upper.3mf");
    verify(
        input,
        "e385145d47ec818770a1b7bb142c6fd6ac5e9f577d89830269f8fcfc79275909",
        json!({}),
        "Bridge line width must not exceed nozzle diameter",
    );
}

pub(super) fn verify(input: &[u8], hash: &str, fields: Value, expected: &str) {
    assert_eq!(super::sha256(input), hash);
    let directory = super::evidence_directory("print-bridge-width-");
    let path = directory.join("input.3mf");
    fs::write(&path, input).unwrap();
    let overrides = ConfigOverrides::from_json(fields.to_string().as_bytes()).unwrap();
    let oracle = OrcaRunner::from_env().unwrap().map(|runner| {
        let output = directory.join("orca");
        fs::create_dir(&output).unwrap();
        let mut args = Vec::new();
        for (key, value) in fields.as_object().unwrap() {
            args.push(format!("--{}", key.replace('_', "-")));
            if value != "1" || !matches!(key.as_str(), "thick_bridges" | "thick_internal_bridges") {
                args.push(value.as_str().unwrap().to_owned());
            }
        }
        args.extend([
            "--slice".to_owned(),
            "0".to_owned(),
            "--outputdir".to_owned(),
            output.to_str().unwrap().to_owned(),
            path.to_str().unwrap().to_owned(),
        ]);
        let error = stages::run(runner.bin(), args, stages::Stage::Slice, &directory).unwrap_err();
        let captured = error.captured.as_ref().unwrap();
        assert_eq!(
            captured.code,
            Some(if cfg!(unix) { 205 } else { -51 }),
            "{error}"
        );
        assert_eq!(
            captured.stderr,
            format!("{expected}\n").as_bytes(),
            "{error}"
        );
        assert!(!output.join("plate_1.gcode").exists());
        json!({"code":captured.code,"stderr":expected,"evidence":error.evidence})
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
    fs::write(directory.join("result.json"),serde_json::to_vec_pretty(&json!({"input_sha256":hash,"overrides":fields,"expected_error":expected,"oracle":oracle,"validation":validation.as_ref().map_err(ToString::to_string),"validated_slice":sliced.as_ref().map(Vec::len).map_err(ToString::to_string)})).unwrap()).unwrap();
    assert!(
        matches!(validation,Err(SliceError::Validate { message: ref text, .. }) if text==expected),
        "{validation:?}; {directory:?}"
    );
    assert!(
        matches!(sliced,Err(SliceError::Validate { message: ref text, .. }) if text==expected),
        "{sliced:?}; {directory:?}"
    );
    assert!(!directory.join("ares.gcode").exists());
}
