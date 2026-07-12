use super::*;
use crate::gcode::format_gcode;
use serde_json::{Value, json};

#[test]
fn pipeline_rejects_invalid_support_placement_before_model_loading() {
    for (key, value) in [
        ("support_object_xy_distance", json!(-0.001)),
        ("support_object_xy_distance", json!(10.001)),
        ("support_object_first_layer_gap", json!("invalid")),
        ("support_object_first_layer_gap", json!(10.001)),
        ("support_on_build_plate_only", json!("true")),
        ("support_critical_regions_only", json!(1)),
        ("support_remove_small_overhang", Value::Null),
    ] {
        let options = options(json!({ key: value }));
        let err = run_slicing_pipeline(b"not a model", &options).unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains(key));
    }
}

#[test]
fn valid_support_placement_values_preserve_current_pipeline_artifacts() {
    let baseline_options = options(json!({
        "support_object_xy_distance": 0.35,
        "support_object_first_layer_gap": 0.2,
        "support_on_build_plate_only": false,
        "support_critical_regions_only": false,
        "support_remove_small_overhang": true
    }));
    let baseline = run_slicing_pipeline(square_pyramid_ascii_stl(), &baseline_options).unwrap();
    let baseline_gcode =
        String::from_utf8(format_gcode(&baseline, &baseline_options).unwrap()).unwrap();

    let tuned_options = options(json!({
        "support_object_xy_distance": 10.0,
        "support_object_first_layer_gap": 0.0,
        "support_on_build_plate_only": true,
        "support_critical_regions_only": true,
        "support_remove_small_overhang": false
    }));
    let tuned = run_slicing_pipeline(square_pyramid_ascii_stl(), &tuned_options).unwrap();
    let tuned_gcode = String::from_utf8(format_gcode(&tuned, &tuned_options).unwrap()).unwrap();

    assert_eq!(tuned.layer_print_paths(), baseline.layer_print_paths());
    assert_eq!(
        tuned.layer_toolpath_moves(),
        baseline.layer_toolpath_moves()
    );
    assert_eq!(
        tuned.layer_extrusion_moves(),
        baseline.layer_extrusion_moves()
    );
    assert_eq!(tuned.layer_speed_moves(), baseline.layer_speed_moves());
    assert_eq!(tuned.diagnostics(), baseline.diagnostics());
    assert_eq!(tuned_gcode, baseline_gcode);
}

fn options(extra: Value) -> SliceOptions {
    serde_json::from_value(extra).unwrap()
}
