//! Native current-bed-key controls, prewritten before the guard port.
use std::fs;

use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use serde_json::json;

use crate::runner::{OrcaRunner, stages};

fn zero(bed: &str, key: &str) {
    super::bed_compatibility::verify(
        super::bed_compatibility::INPUT,
        super::bed_compatibility::INPUT_HASH,
        json!({"curr_bed_type":bed,key:["0"]}),
        &format!("Plate 1: {bed} does not support filament 1\n"),
    );
}

#[test]
fn print_bed_compat_native_textured_cool_key() {
    zero("Textured Cool Plate", "textured_cool_plate_temp");
}
#[test]
fn print_bed_compat_native_engineering_key() {
    zero("Engineering Plate", "eng_plate_temp");
}
#[test]
fn print_bed_compat_native_high_temp_key() {
    zero("High Temp Plate", "hot_plate_temp");
}
#[test]
fn print_bed_compat_native_textured_pei_key() {
    zero("Textured PEI Plate", "textured_plate_temp");
}
#[test]
fn print_bed_compat_native_supertack_enum_key_not_ui_label() {
    zero("Supertack Plate", "supertack_plate_temp");
}

#[test]
fn print_bed_compat_native_multi_sorted_first_zero_is_index_zero() {
    super::bed_compatibility::verify(
        super::bed_compatibility::MULTI_INPUT,
        super::bed_compatibility::MULTI_HASH,
        json!({"hot_plate_temp":["0","0"]}),
        "Plate 1: High Temp Plate does not support filament 1\n",
    );
}

#[test]
fn print_bed_compat_native_selected_second_plate_reports_plate_two() {
    let input = include_bytes!("../../../../../tests/parity/validation/print-bed-two-plates.3mf");
    let hash = "a5aa1fafb4ed2daffa2a496f4e8853b559fe0710be09a13c77da03a13f653a07";
    assert_eq!(super::sha256(input), hash);
    let project = ares_core::load_project(input).unwrap();
    assert_eq!(project.plates().len(), 2);
    assert_eq!(
        project
            .plates()
            .iter()
            .map(|plate| plate.id())
            .collect::<Vec<_>>(),
        [1, 2]
    );
    let expected = "Plate 2: High Temp Plate does not support filament 1\n";
    let directory = super::evidence_directory("print-bed-plate-two-");
    let path = directory.join("input.3mf");
    fs::write(&path, input).unwrap();
    let oracle = OrcaRunner::from_env().unwrap().map(|runner| {
        let output = directory.join("orca");
        fs::create_dir(&output).unwrap();
        let error = stages::run(runner.bin(), [
            "--hot-plate-temp", "0", "--slice", "2", "--outputdir",
            output.to_str().unwrap(), path.to_str().unwrap(),
        ].map(str::to_owned), stages::Stage::Slice, &directory).unwrap_err();
        let captured = error.captured.as_ref().unwrap();
        assert_eq!(captured.code, Some(if cfg!(unix) {195} else {-61}), "{error}");
        assert_eq!(captured.stderr, format!("{expected}\n").as_bytes(), "{error}");
        assert!(!output.join("plate_2.gcode").exists());
        json!({"code":captured.code,"stderr":String::from_utf8(captured.stderr.clone()).unwrap(),"evidence":error.evidence})
    });
    let overrides = ConfigOverrides::from_json(br#"{"hot_plate_temp":["0"]}"#).unwrap();
    let validation = ares_core::validate_project_plate(input, Some(2), &overrides);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let sliced = runtime.block_on(crate::runner::application::slice_project_plate_validated(
        input,
        GenerationMetadata::default(),
        Some(2),
        &overrides,
        SlicerContext::new(0),
    ));
    fs::write(directory.join("result.json"),serde_json::to_vec_pretty(&json!({"input_sha256":hash,"selected_plate":2,"oracle":oracle,"expected_error":expected,"validation":validation.as_ref().map_err(ToString::to_string),"validated_slice":sliced.as_ref().map(Vec::len).map_err(ToString::to_string)})).unwrap()).unwrap();
    assert!(
        matches!(validation, Err(ares_core::SliceError::Validate { message: ref text, .. }) if text==expected),
        "{validation:?}; {directory:?}"
    );
}

#[test]
fn print_bed_compat_native_initial_layer_zero_and_original_still_slice() {
    let input = super::bed_compatibility::INPUT;
    for fields in [json!({}), json!({"cool_plate_temp_initial_layer":["0"]})] {
        let directory = super::evidence_directory("print-bed-positive-");
        let path = directory.join("input.3mf");
        fs::write(&path, input).unwrap();
        let oracle = OrcaRunner::from_env().unwrap().map(|runner| {
            let output = directory.join("orca");
            fs::create_dir(&output).unwrap();
            let mut args = Vec::new();
            if !fields.as_object().unwrap().is_empty() {
                args.extend(["--cool-plate-temp-initial-layer".to_owned(), "0".to_owned()]);
            }
            args.extend([
                "--slice".to_owned(),
                "0".to_owned(),
                "--outputdir".to_owned(),
                output.to_str().unwrap().to_owned(),
                path.to_str().unwrap().to_owned(),
            ]);
            stages::run(runner.bin(), args, stages::Stage::Slice, &directory).unwrap();
            let gcode = fs::read(output.join("plate_1.gcode")).unwrap();
            assert!(!gcode.is_empty());
            json!({"status":"GCODE","sha256":super::sha256(&gcode)})
        });
        let overrides = ConfigOverrides::from_json(fields.to_string().as_bytes()).unwrap();
        let validation = ares_core::validate_project_plate(input, None, &overrides);
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let sliced = runtime.block_on(crate::runner::application::slice_project_plate_validated(
            input,
            GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &overrides,
            SlicerContext::new(0),
        ));
        if let Ok(gcode) = &sliced {
            fs::write(directory.join("ares.gcode"), gcode).unwrap();
        }
        fs::write(directory.join("result.json"),serde_json::to_vec_pretty(&json!({"overrides":fields,"oracle":oracle,"validation":validation.as_ref().map_err(ToString::to_string),"validated_slice":sliced.as_ref().map(Vec::len).map_err(ToString::to_string)})).unwrap()).unwrap();
        assert!(validation.is_ok(), "{validation:?}; {directory:?}");
        assert!(
            sliced.as_ref().is_ok_and(|gcode| !gcode.is_empty()),
            "{sliced:?}; {directory:?}"
        );
    }
}
