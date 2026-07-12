use super::*;
use crate::gcode::format_gcode;

#[test]
fn pipeline_rejects_invalid_support_threshold_before_model_loading() {
    for (key, value) in [
        ("independent_support_layer_height", json!("true")),
        ("support_threshold_angle", json!(91)),
        ("support_threshold_angle", json!(45.5)),
        ("support_threshold_overlap", json!(100.001)),
        ("support_threshold_overlap", json!("100.001%")),
        ("support_threshold_overlap", json!("bad%")),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({ key: value })).unwrap();
        let err = run_slicing_pipeline(b"not a model", &options).unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains(key));
    }
}

#[test]
fn valid_support_threshold_values_preserve_current_pipeline_artifacts() {
    let baseline_options: SliceOptions = serde_json::from_value(json!({
        "independent_support_layer_height": true,
        "support_threshold_angle": 30,
        "support_threshold_overlap": "50%"
    }))
    .unwrap();
    let baseline = run_slicing_pipeline(square_pyramid_ascii_stl(), &baseline_options).unwrap();
    let baseline_gcode =
        String::from_utf8(format_gcode(&baseline, &baseline_options).unwrap()).unwrap();

    let tuned_options: SliceOptions = serde_json::from_value(json!({
        "independent_support_layer_height": false,
        "support_threshold_angle": 0,
        "support_threshold_overlap": "25%"
    }))
    .unwrap();
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
