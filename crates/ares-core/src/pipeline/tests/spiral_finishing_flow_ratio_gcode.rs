use crate::{
    SliceError, SliceOptions, ToolpathMoveKind, gcode::format_gcode,
    pipeline::test_support::rectangular_layers_pipeline,
};
use serde_json::json;

#[test]
fn spiral_finishing_flow_ratio_zero_appends_tapered_final_layer_after_normal_output() {
    let (output, normal_count) =
        spiral_gcode_with_final_layer_print_count(json!({ "spiral_finishing_flow_ratio": 0.0 }))
            .unwrap();
    let final_layer = layer_extrusions(&output, 2);

    assert!(normal_count >= 3);
    assert!(final_layer.len() > normal_count);
    let normal = &final_layer[..normal_count];
    let transition = &final_layer[normal_count..];

    assert!(transition.iter().all(|value| *value >= 0.0));
    assert!(
        transition
            .iter()
            .zip(normal)
            .all(|(tapered, base)| tapered <= base)
    );
    assert!(
        transition
            .iter()
            .zip(normal)
            .any(|(tapered, base)| tapered < base)
    );
}

#[test]
fn spiral_finishing_flow_ratio_one_appends_duplicate_final_layer_e_values() {
    let (output, normal_count) =
        spiral_gcode_with_final_layer_print_count(json!({ "spiral_finishing_flow_ratio": 1.0 }))
            .unwrap();
    let final_layer = layer_extrusions(&output, 2);

    assert_eq!(final_layer.len(), normal_count * 2);
    assert_eq!(&final_layer[..normal_count], &final_layer[normal_count..]);
}

#[test]
fn spiral_finishing_flow_ratio_is_ignored_for_absolute_e() {
    let baseline = spiral_gcode(json!({
        "spiral_finishing_flow_ratio": 1.0,
        "use_relative_e_distances": false
    }))
    .unwrap();
    let tapered = spiral_gcode(json!({
        "spiral_finishing_flow_ratio": 0.0,
        "use_relative_e_distances": false
    }))
    .unwrap();

    assert_eq!(
        layer_extrusions(&tapered, 2),
        layer_extrusions(&baseline, 2)
    );
}

#[test]
fn spiral_finishing_flow_ratio_is_ignored_when_spiral_mode_is_disabled() {
    let baseline = spiral_gcode(json!({
        "spiral_mode": false,
        "spiral_finishing_flow_ratio": 1.0
    }))
    .unwrap();
    let tapered = spiral_gcode(json!({
        "spiral_mode": false,
        "spiral_finishing_flow_ratio": 0.0
    }))
    .unwrap();

    assert_eq!(
        layer_extrusions(&tapered, 2),
        layer_extrusions(&baseline, 2)
    );
}

#[test]
fn spiral_finishing_flow_ratio_rejects_invalid_values() {
    for value in [json!(-0.01), json!(1.25), json!("NaN")] {
        let err = spiral_gcode(json!({ "spiral_finishing_flow_ratio": value })).unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("spiral_finishing_flow_ratio"));
    }
}

fn spiral_gcode(extra: serde_json::Value) -> Result<String, SliceError> {
    spiral_gcode_with_final_layer_print_count(extra).map(|(output, _)| output)
}

fn spiral_gcode_with_final_layer_print_count(
    extra: serde_json::Value,
) -> Result<(String, usize), SliceError> {
    let mut options = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 3,
        "line_width": 0.4,
        "sparse_infill_density": 50,
        "minimum_sparse_infill_area": 0,
        "bottom_shell_layers": 1,
        "top_shell_layers": 3,
        "spiral_mode": true,
        "seam_gap": 0
    });
    for (key, value) in extra.as_object().unwrap() {
        options[key] = value.clone();
    }
    let mut options: SliceOptions = serde_json::from_value(options).unwrap();
    options.normalize_fdm(0)?;
    let pipeline = rectangular_layers_pipeline(&options, 3);
    let final_layer_print_count = pipeline.layer_extrusion_moves()[2]
        .moves()
        .iter()
        .filter(|move_| move_.kind() == ToolpathMoveKind::Print)
        .count();
    let output = format_gcode(&pipeline, &options)?;
    Ok((String::from_utf8(output).unwrap(), final_layer_print_count))
}

fn layer_extrusions(output: &str, layer: usize) -> Vec<f64> {
    let mut in_layer = false;
    let marker = format!(";LAYER:{layer}");
    output
        .lines()
        .filter_map(|line| {
            if line == marker {
                in_layer = true;
                return None;
            }
            if in_layer && line.starts_with(";LAYER:") {
                in_layer = false;
            }
            (in_layer && line.starts_with("G1 X") && line.contains(" E"))
                .then(|| e_value(line))
                .flatten()
        })
        .collect()
}

fn e_value(line: &str) -> Option<f64> {
    line.split_whitespace()
        .find_map(|word| word.strip_prefix('E'))
        .and_then(|value| value.parse().ok())
}
