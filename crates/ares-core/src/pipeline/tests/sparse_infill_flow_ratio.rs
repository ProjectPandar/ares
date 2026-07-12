use crate::{SliceOptions, gcode::format_gcode, pipeline::test_support::rectangular_pipeline};
use serde_json::json;

#[test]
fn sparse_infill_flow_ratio_changes_sparse_infill_gcode_extrusion_delta() {
    let low_flow: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 0,
        "skirt_loops": 0,
        "brim_width": 0.0,
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.4,
        "minimum_sparse_infill_area": 0,
        "set_other_flow_ratios": true,
        "sparse_infill_flow_ratio": 0.5,
        "line_width": 0.4,
        "bottom_shell_layers": 0,
        "top_shell_layers": 0
    }))
    .unwrap();
    let high_flow: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 0,
        "skirt_loops": 0,
        "brim_width": 0.0,
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.4,
        "minimum_sparse_infill_area": 0,
        "set_other_flow_ratios": true,
        "sparse_infill_flow_ratio": 1.5,
        "line_width": 0.4,
        "bottom_shell_layers": 0,
        "top_shell_layers": 0
    }))
    .unwrap();
    let low_pipeline = rectangular_pipeline(&low_flow);
    let high_pipeline = rectangular_pipeline(&high_flow);

    assert_eq!(
        low_pipeline.diagnostics().total_infill_count(),
        high_pipeline.diagnostics().total_infill_count()
    );

    let low_gcode = String::from_utf8(format_gcode(&low_pipeline, &low_flow).unwrap()).unwrap();
    let high_gcode = String::from_utf8(format_gcode(&high_pipeline, &high_flow).unwrap()).unwrap();
    let low_delta = first_sparse_infill_extrusion_delta(&low_gcode);
    let high_delta = first_sparse_infill_extrusion_delta(&high_gcode);

    assert!((high_delta - low_delta * 3.0).abs() <= 0.000002);
}

fn first_sparse_infill_extrusion_delta(gcode: &str) -> f64 {
    let mut previous_e = 0.0;
    for line in gcode.lines() {
        if let Some(e) = line
            .strip_prefix(";EXTRUSION:print:")
            .and_then(|line| line.rsplit_once(':').map(|(_, e)| e))
            .and_then(|e| e.parse::<f64>().ok())
        {
            if line.starts_with(";EXTRUSION:print:sparse_infill:") {
                return e - previous_e;
            }
            previous_e = e;
        }
    }
    panic!("missing sparse infill extrusion");
}
