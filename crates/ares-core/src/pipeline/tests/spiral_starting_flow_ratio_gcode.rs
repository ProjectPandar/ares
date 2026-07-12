use crate::{
    SliceError, SliceOptions, gcode::format_gcode,
    pipeline::test_support::rectangular_layers_pipeline,
};
use serde_json::json;

#[test]
fn spiral_starting_flow_ratio_tapers_first_body_layer_relative_e_without_catchup() {
    let baseline = spiral_gcode(json!({ "spiral_starting_flow_ratio": 1.0 })).unwrap();
    let tapered = spiral_gcode(json!({ "spiral_starting_flow_ratio": 0.0 })).unwrap();

    assert_eq!(
        layer_extrusions(&tapered, 0),
        layer_extrusions(&baseline, 0)
    );
    let baseline_layer = layer_extrusions(&baseline, 1);
    let tapered_layer = layer_extrusions(&tapered, 1);
    assert_eq!(
        layer_extrusions(&tapered, 2),
        layer_extrusions(&baseline, 2)
    );

    assert!(baseline_layer.len() >= 3);
    assert_eq!(baseline_layer.len(), tapered_layer.len());
    assert!(tapered_layer[0] < baseline_layer[0]);
    assert!(tapered_layer[1] > tapered_layer[0]);
    assert!(tapered_layer[1] < baseline_layer[1]);
    assert_eq!(tapered_layer.last(), baseline_layer.last());
}

#[test]
fn spiral_starting_flow_ratio_uses_normalized_bottom_shell_layer_as_transition_index() {
    let baseline = spiral_gcode_with_layers(
        json!({
            "bottom_shell_layers": 2,
            "spiral_starting_flow_ratio": 1.0
        }),
        4,
    )
    .unwrap();
    let tapered = spiral_gcode_with_layers(
        json!({
            "bottom_shell_layers": 2,
            "spiral_starting_flow_ratio": 0.0
        }),
        4,
    )
    .unwrap();

    assert_eq!(
        layer_extrusions(&tapered, 0),
        layer_extrusions(&baseline, 0)
    );
    assert_eq!(
        layer_extrusions(&tapered, 1),
        layer_extrusions(&baseline, 1)
    );
    assert_ne!(
        layer_extrusions(&tapered, 2),
        layer_extrusions(&baseline, 2)
    );
    assert_eq!(
        layer_extrusions(&tapered, 3),
        layer_extrusions(&baseline, 3)
    );
}

#[test]
fn spiral_starting_flow_ratio_is_ignored_for_absolute_e() {
    let baseline = spiral_gcode(json!({
        "spiral_starting_flow_ratio": 1.0,
        "use_relative_e_distances": false
    }))
    .unwrap();
    let tapered = spiral_gcode(json!({
        "spiral_starting_flow_ratio": 0.0,
        "use_relative_e_distances": false
    }))
    .unwrap();

    assert_eq!(
        layer_extrusions(&tapered, 1),
        layer_extrusions(&baseline, 1)
    );
}

#[test]
fn spiral_starting_flow_ratio_is_ignored_when_spiral_mode_is_disabled() {
    let baseline = spiral_gcode(json!({
        "spiral_mode": false,
        "spiral_starting_flow_ratio": 1.0
    }))
    .unwrap();
    let tapered = spiral_gcode(json!({
        "spiral_mode": false,
        "spiral_starting_flow_ratio": 0.0
    }))
    .unwrap();

    for layer in 0..3 {
        assert_eq!(
            layer_extrusions(&tapered, layer),
            layer_extrusions(&baseline, layer)
        );
    }
}

#[test]
fn spiral_starting_flow_ratio_rejects_invalid_values() {
    for value in [json!(-0.01), json!(1.25), json!("NaN")] {
        let err = spiral_gcode(json!({ "spiral_starting_flow_ratio": value })).unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("spiral_starting_flow_ratio"));
    }
}

fn spiral_gcode(extra: serde_json::Value) -> Result<String, SliceError> {
    spiral_gcode_with_layers(extra, 3)
}

fn spiral_gcode_with_layers(
    extra: serde_json::Value,
    layer_count: usize,
) -> Result<String, SliceError> {
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
    let pipeline = rectangular_layers_pipeline(&options, layer_count);
    let output = format_gcode(&pipeline, &options)?;
    Ok(String::from_utf8(output).unwrap())
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
