use super::*;
use crate::gcode::format_gcode;

fn options(value: &str) -> SliceOptions {
    serde_json::from_value(json!({ "support_type": value })).unwrap()
}

#[test]
fn pipeline_rejects_invalid_support_type_before_model_loading() {
    let options = options("invalid");

    let err = run_slicing_pipeline(b"not a model", &options).unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("support_type"));
}

#[test]
fn valid_support_type_values_preserve_current_pipeline_artifacts() {
    let baseline_options = options("normal(auto)");
    let baseline = run_slicing_pipeline(square_pyramid_ascii_stl(), &baseline_options).unwrap();
    let baseline_gcode =
        String::from_utf8(format_gcode(&baseline, &baseline_options).unwrap()).unwrap();

    for value in [
        "normal(auto)",
        "tree(auto)",
        "normal(manual)",
        "tree(manual)",
    ] {
        let options = options(value);
        let pipeline = run_slicing_pipeline(square_pyramid_ascii_stl(), &options).unwrap();
        let gcode = String::from_utf8(format_gcode(&pipeline, &options).unwrap()).unwrap();

        assert_eq!(
            pipeline.layer_print_paths(),
            baseline.layer_print_paths(),
            "{value}"
        );
        assert_eq!(
            pipeline.layer_toolpath_moves(),
            baseline.layer_toolpath_moves(),
            "{value}"
        );
        assert_eq!(
            pipeline.layer_extrusion_moves(),
            baseline.layer_extrusion_moves(),
            "{value}"
        );
        assert_eq!(
            pipeline.layer_speed_moves(),
            baseline.layer_speed_moves(),
            "{value}"
        );
        assert_eq!(gcode, baseline_gcode, "{value}");
    }
}
