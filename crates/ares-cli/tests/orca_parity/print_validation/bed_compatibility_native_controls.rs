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
        let sliced = runtime.block_on(ares_core::slice_project_plate_validated(
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
