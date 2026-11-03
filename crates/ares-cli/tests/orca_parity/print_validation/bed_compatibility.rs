//! Print.cpp:1751-1781: ordinary bed temperature compatibility, source newline.
use std::fs;

use ares_core::{ConfigOverrides, GenerationMetadata, SliceError, SlicerContext};
use serde_json::{Value, json};

use crate::runner::{OrcaRunner, stages};

pub(super) const INPUT: &[u8] =
    include_bytes!("../../../../../tests/parity/validation/e5max-scv/project.3mf");
pub(super) const INPUT_HASH: &str =
    "c503c92c8f524e16b4223d6349b572f333634bac418b7874236e64ceb536eaff";
pub(super) const MULTI_INPUT: &[u8] =
    include_bytes!("../../../../../tests/parity/validation/print-bed-multi-used.3mf");
pub(super) const MULTI_HASH: &str =
    "287d114401111f697b74ab622c63717c891737a9493da2ccc513d076ed983f33";

#[test]
fn print_bed_compat_original_cool_zero_rejects_before_processing() {
    verify(
        INPUT,
        INPUT_HASH,
        json!({"cool_plate_temp":["0"]}),
        "Plate 1: Cool Plate does not support filament 1\n",
    );
}

#[test]
fn print_bed_compat_multi_used_first_failure_is_logical_index_one() {
    // Original KSR geometry/settings/templates unchanged; one object metadata
    // sparse_infill_filament_id=2 joins walls' filament1 in Print::extruders().
    verify(
        MULTI_INPUT,
        MULTI_HASH,
        json!({"hot_plate_temp":["55","0"]}),
        "Plate 1: High Temp Plate does not support filament 2\n",
    );
}

pub(super) fn verify(input: &[u8], hash: &str, fields: Value, expected: &str) {
    assert_eq!(super::sha256(input), hash);
    let directory = super::evidence_directory("print-bed-compat-");
    let path = directory.join("input.3mf");
    fs::write(&path, input).unwrap();
    let overrides = ConfigOverrides::from_json(fields.to_string().as_bytes()).unwrap();
    let oracle = OrcaRunner::from_env().unwrap().map(|runner| {
        let output = directory.join("orca");
        fs::create_dir(&output).unwrap();
        let mut args = Vec::new();
        for (key, value) in fields.as_object().unwrap() {
            args.push(format!("--{}", key.replace('_', "-")));
            args.push(match value {
                Value::String(token) => token.clone(),
                Value::Array(values) => values.iter().map(|v| v.as_str().unwrap()).collect::<Vec<_>>().join(","),
                _ => unreachable!("native bed probe uses string/scalar vectors"),
            });
        }
        args.extend([
            "--slice".to_owned(), "0".to_owned(),
            "--outputdir".to_owned(), output.to_str().unwrap().to_owned(),
            path.to_str().unwrap().to_owned(),
        ]);
        let error = stages::run(runner.bin(), args, stages::Stage::Slice, &directory).unwrap_err();
        let captured = error.captured.as_ref().unwrap();
        assert_eq!(captured.code, Some(if cfg!(unix) { 195 } else { -61 }), "{error}");
        assert_eq!(captured.stderr, format!("{expected}\n").as_bytes(), "{error}");
        assert!(!output.join("plate_1.gcode").exists());
        json!({"code":captured.code,"stderr":String::from_utf8(captured.stderr.clone()).unwrap(),"evidence":error.evidence})
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
            "input_sha256":hash,"overrides":fields,"expected_error":expected,"oracle":oracle,
            "validation":validation.as_ref().map_err(ToString::to_string),
            "validated_slice":sliced.as_ref().map(Vec::len).map_err(ToString::to_string),
        }))
        .unwrap(),
    )
    .unwrap();
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
