use crate::{SliceOptions, gcode::format_gcode, pipeline::test_support::rectangular_pipeline};
use serde_json::{Value, json};

#[test]
fn set_other_flow_ratios_gates_wall_gcode_extrusion_deltas() {
    let omitted = wall_options(None);
    let disabled = wall_options(Some(false));
    let enabled = wall_options(Some(true));
    let omitted_pipeline = rectangular_pipeline(&omitted);
    let disabled_pipeline = rectangular_pipeline(&disabled);
    let enabled_pipeline = rectangular_pipeline(&enabled);

    assert_eq!(
        omitted_pipeline.diagnostics().total_perimeter_count(),
        enabled_pipeline.diagnostics().total_perimeter_count()
    );
    assert_eq!(
        disabled_pipeline.diagnostics().total_perimeter_count(),
        enabled_pipeline.diagnostics().total_perimeter_count()
    );

    let omitted_gcode =
        String::from_utf8(format_gcode(&omitted_pipeline, &omitted).unwrap()).unwrap();
    let disabled_gcode =
        String::from_utf8(format_gcode(&disabled_pipeline, &disabled).unwrap()).unwrap();
    let enabled_gcode =
        String::from_utf8(format_gcode(&enabled_pipeline, &enabled).unwrap()).unwrap();

    assert_delta_eq(
        first_extrusion_delta(&omitted_gcode, "external_perimeter"),
        first_extrusion_delta(&disabled_gcode, "external_perimeter"),
    );
    assert_delta_eq(
        first_extrusion_delta(&omitted_gcode, "internal_perimeter"),
        first_extrusion_delta(&disabled_gcode, "internal_perimeter"),
    );
    assert_delta_eq(
        first_extrusion_delta(&enabled_gcode, "external_perimeter"),
        first_extrusion_delta(&disabled_gcode, "external_perimeter") * 0.5,
    );
    assert_delta_eq(
        first_extrusion_delta(&enabled_gcode, "internal_perimeter"),
        first_extrusion_delta(&disabled_gcode, "internal_perimeter") * 0.75,
    );
}

#[test]
fn set_other_flow_ratios_gates_first_layer_gcode_extrusion_delta() {
    let omitted = first_layer_options(None);
    let enabled = first_layer_options(Some(true));
    let omitted_gcode =
        String::from_utf8(format_gcode(&rectangular_pipeline(&omitted), &omitted).unwrap())
            .unwrap();
    let enabled_gcode =
        String::from_utf8(format_gcode(&rectangular_pipeline(&enabled), &enabled).unwrap())
            .unwrap();

    assert_delta_eq(
        first_extrusion_delta(&enabled_gcode, "external_perimeter"),
        first_extrusion_delta(&omitted_gcode, "external_perimeter") * 0.5,
    );
}

#[test]
fn set_other_flow_ratios_gates_sparse_infill_gcode_extrusion_delta() {
    let omitted = sparse_infill_options(None);
    let enabled = sparse_infill_options(Some(true));
    let omitted_pipeline = rectangular_pipeline(&omitted);
    let enabled_pipeline = rectangular_pipeline(&enabled);

    assert_eq!(
        omitted_pipeline.diagnostics().total_infill_count(),
        enabled_pipeline.diagnostics().total_infill_count()
    );

    let omitted_gcode =
        String::from_utf8(format_gcode(&omitted_pipeline, &omitted).unwrap()).unwrap();
    let enabled_gcode =
        String::from_utf8(format_gcode(&enabled_pipeline, &enabled).unwrap()).unwrap();

    assert_delta_eq(
        first_extrusion_delta(&enabled_gcode, "sparse_infill"),
        first_extrusion_delta(&omitted_gcode, "sparse_infill") * 0.5,
    );
}

fn wall_options(gate: Option<bool>) -> SliceOptions {
    options_with_gate(
        json!({
            "layer_height": 0.2,
            "initial_layer_height": 0.2,
            "wall_loops": 2,
            "line_width": 0.4,
            "sparse_infill_density": 0,
            "outer_wall_flow_ratio": 0.5,
            "inner_wall_flow_ratio": 0.75
        }),
        gate,
    )
}

fn first_layer_options(gate: Option<bool>) -> SliceOptions {
    options_with_gate(
        json!({
            "layer_height": 0.2,
            "initial_layer_height": 0.2,
            "wall_loops": 1,
            "line_width": 0.4,
            "sparse_infill_density": 0,
            "skirt_loops": 0,
            "first_layer_flow_ratio": 0.5
        }),
        gate,
    )
}

fn sparse_infill_options(gate: Option<bool>) -> SliceOptions {
    options_with_gate(
        json!({
            "layer_height": 0.2,
            "initial_layer_height": 0.2,
            "wall_loops": 0,
            "skirt_loops": 0,
            "brim_width": 0.0,
            "sparse_infill_density": 50,
            "sparse_infill_line_width": 0.4,
            "minimum_sparse_infill_area": 0,
            "sparse_infill_flow_ratio": 0.5,
            "line_width": 0.4,
            "bottom_shell_layers": 0,
            "top_shell_layers": 0
        }),
        gate,
    )
}

fn options_with_gate(mut value: Value, gate: Option<bool>) -> SliceOptions {
    if let Some(gate) = gate {
        value["set_other_flow_ratios"] = json!(gate);
    }
    serde_json::from_value(value).unwrap()
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

fn assert_delta_eq(actual: f64, expected: f64) {
    assert!((actual - expected).abs() <= 0.000002);
}
