use crate::{SliceOptions, gcode::format_gcode, pipeline::test_support::rectangular_pipeline};
use serde_json::json;

#[test]
fn wall_flow_ratios_change_perimeter_gcode_extrusion_deltas() {
    let low_flow: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 2,
        "line_width": 0.4,
        "sparse_infill_density": 0,
        "set_other_flow_ratios": true,
        "outer_wall_flow_ratio": 0.5,
        "inner_wall_flow_ratio": 0.75
    }))
    .unwrap();
    let high_flow: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 2,
        "line_width": 0.4,
        "sparse_infill_density": 0,
        "set_other_flow_ratios": true,
        "outer_wall_flow_ratio": 1.5,
        "inner_wall_flow_ratio": 1.25
    }))
    .unwrap();
    let low_pipeline = rectangular_pipeline(&low_flow);
    let high_pipeline = rectangular_pipeline(&high_flow);

    assert_eq!(
        low_pipeline.diagnostics().total_perimeter_count(),
        high_pipeline.diagnostics().total_perimeter_count()
    );

    let low_gcode = String::from_utf8(format_gcode(&low_pipeline, &low_flow).unwrap()).unwrap();
    let high_gcode = String::from_utf8(format_gcode(&high_pipeline, &high_flow).unwrap()).unwrap();
    let low_external = first_extrusion_delta(&low_gcode, "external_perimeter");
    let high_external = first_extrusion_delta(&high_gcode, "external_perimeter");
    let low_internal = first_extrusion_delta(&low_gcode, "internal_perimeter");
    let high_internal = first_extrusion_delta(&high_gcode, "internal_perimeter");

    assert!((high_external - low_external * 3.0).abs() <= 0.000002);
    assert!((high_internal - low_internal * (1.25 / 0.75)).abs() <= 0.000002);
}

fn first_extrusion_delta(gcode: &str, role: &str) -> f64 {
    let mut previous_e = 0.0;
    let target = format!(";EXTRUSION:print:{role}:");
    for line in gcode.lines() {
        if let Some(e) = line
            .strip_prefix(";EXTRUSION:print:")
            .and_then(|line| line.rsplit_once(':').map(|(_, e)| e))
            .and_then(|e| e.parse::<f64>().ok())
        {
            if line.starts_with(&target) {
                return e - previous_e;
            }
            previous_e = e;
        }
    }
    panic!("missing {role} extrusion");
}
