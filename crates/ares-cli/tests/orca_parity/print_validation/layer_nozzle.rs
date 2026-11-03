//! Print.cpp:1627-1652, Config.hpp:624-628: public layer-height/raft errors.
use std::fs;

use ares_core::{ConfigOverrides, GenerationMetadata, SliceError, SlicerContext};
use serde_json::{Value, json};

use crate::runner::{OrcaRunner, stages};

#[test]
fn print_layer_nozzle_object_height_precedes_width() {
    verify(json!({"layer_height":"0.3"}));
}

#[test]
fn print_layer_nozzle_first_height_precedes_width() {
    verify(json!({"initial_layer_print_height":"0.3"}));
}

#[test]
fn print_layer_nozzle_raft_support_first_height_precedes_width() {
    verify(json!({"raft_layers":"2","support_filament":"1","initial_layer_print_height":"0.3"}));
}

#[test]
fn print_layer_nozzle_raft_two_logical_filaments_share_one_physical_tool() {
    verify(
        json!({"raft_layers":"2","support_filament":"2","support_interface_filament":"2","filament_diameter":["1.75","1.75"],"filament_map":["1","1"],"initial_layer_print_height":"0.3"}),
    );
}

pub(super) fn verify(fields: Value) {
    let input =
        include_bytes!("../../../../../tests/parity/validation/minimum-width-ender3s1plus-0.2.3mf");
    let hash = "edb48192a609ede526e8e09e805b845ff99494c0d2d2b31a8655ca9b0ac89110";
    assert_eq!(super::sha256(input), hash);
    let directory = super::evidence_directory("print-layer-nozzle-");
    let path = directory.join("input.3mf");
    fs::write(&path, input).unwrap();
    let overrides = ConfigOverrides::from_json(fields.to_string().as_bytes()).unwrap();
    let runner = OrcaRunner::from_env().unwrap();
    let oracle = runner.as_ref().map(|runner| {
        let output = directory.join("orca");
        fs::create_dir(&output).unwrap();
        let mut arguments = Vec::new();
        for (key,value) in fields.as_object().unwrap() {
            arguments.push(format!("--{}", key.replace('_',"-")));
            arguments.push(match value {
                Value::String(value) => value.clone(),
                Value::Array(values) => values.iter().map(|value| value.as_str().unwrap()).collect::<Vec<_>>().join(","),
                _ => unreachable!(),
            });
        }
        arguments.extend(["--slice".to_owned(),"0".to_owned(),"--outputdir".to_owned(),output.to_str().unwrap().to_owned(),path.to_str().unwrap().to_owned()]);
        let error = stages::run(runner.bin(),arguments,stages::Stage::Slice,&directory).unwrap_err();
        let captured = error.captured.as_ref().unwrap();
        assert_eq!(captured.code,Some(if cfg!(unix) {205} else {-51}),"{error}");
        assert_eq!(captured.stderr,b"Layer height cannot exceed nozzle diameter.\n","{error}");
        assert!(!output.join("plate_1.gcode").exists());
        json!({"code":captured.code,"error":"Layer height cannot exceed nozzle diameter.","evidence":error.evidence})
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
            "input_sha256":hash,"overrides":fields,"oracle":oracle,
            "validation":validation.as_ref().map_err(ToString::to_string),
            "validated_slice":sliced.as_ref().map(Vec::len).map_err(ToString::to_string),
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(
        matches!(validation,Err(SliceError::Validate { message: ref text, .. }) if text=="Layer height cannot exceed nozzle diameter."),
        "{validation:?}; {directory:?}"
    );
    assert!(
        matches!(sliced,Err(SliceError::Validate { message: ref text, .. }) if text=="Layer height cannot exceed nozzle diameter."),
        "validated composition mismatch; {directory:?}"
    );
    assert!(!directory.join("ares.gcode").exists());
}
