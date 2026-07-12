use crate::{SliceOptions, gcode::format_gcode, pipeline::test_support::rectangular_pipeline};
use serde_json::json;

#[test]
fn print_flow_ratio_changes_perimeter_gcode_extrusion_delta() {
    let low_flow: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 1,
        "line_width": 0.4,
        "sparse_infill_density": 0,
        "skirt_loops": 0,
        "brim_width": 0.0,
        "print_flow_ratio": 0.5
    }))
    .unwrap();
    let high_flow: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 1,
        "line_width": 0.4,
        "sparse_infill_density": 0,
        "skirt_loops": 0,
        "brim_width": 0.0,
        "print_flow_ratio": 1.5
    }))
    .unwrap();

    let low_gcode =
        String::from_utf8(format_gcode(&rectangular_pipeline(&low_flow), &low_flow).unwrap())
            .unwrap();
    let high_gcode =
        String::from_utf8(format_gcode(&rectangular_pipeline(&high_flow), &high_flow).unwrap())
            .unwrap();

    assert!(
        (first_extrusion_delta(&high_gcode, "external_perimeter")
            - first_extrusion_delta(&low_gcode, "external_perimeter") * 3.0)
            .abs()
            <= 0.000002
    );
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
