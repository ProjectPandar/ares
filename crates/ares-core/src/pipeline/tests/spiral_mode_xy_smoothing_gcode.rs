use crate::{
    Contour, Point2, SliceError, SliceOptions, gcode::format_gcode,
    pipeline::test_support::contour_layers_pipeline_from_layers_for_tests,
};
use serde_json::json;

#[test]
fn spiral_mode_smooth_adjusts_shifted_second_layer_xy_and_e() {
    let baseline = spiral_gcode(json!({ "spiral_mode_smooth": false })).unwrap();
    let smoothed = spiral_gcode(json!({
        "spiral_mode_smooth": true,
        "spiral_mode_max_xy_smoothing": 10.0
    }))
    .unwrap();

    let baseline_layer = layer_print_moves(&baseline, 1);
    let smoothed_layer = layer_print_moves(&smoothed, 1);

    assert_eq!(smoothed_layer.len(), baseline_layer.len());
    assert!(
        smoothed_layer
            .iter()
            .zip(&baseline_layer)
            .any(|(smoothed, baseline)| smoothed.x < baseline.x)
    );
    assert!(
        smoothed_layer
            .iter()
            .zip(&baseline_layer)
            .any(|(smoothed, baseline)| smoothed.e < baseline.e)
    );
}

#[test]
fn zero_spiral_mode_max_xy_smoothing_preserves_unsmoothed_xy() {
    let baseline = spiral_gcode(json!({ "spiral_mode_smooth": false })).unwrap();
    let zero_threshold = spiral_gcode(json!({
        "spiral_mode_smooth": true,
        "spiral_mode_max_xy_smoothing": 0.0
    }))
    .unwrap();

    assert_eq!(
        layer_print_moves(&zero_threshold, 1),
        layer_print_moves(&baseline, 1)
    );
}

#[test]
fn spiral_mode_smooth_applies_with_absolute_e_distances() {
    let baseline = spiral_gcode(json!({
        "spiral_mode_smooth": false,
        "use_relative_e_distances": false
    }))
    .unwrap();
    let smoothed = spiral_gcode(json!({
        "spiral_mode_smooth": true,
        "spiral_mode_max_xy_smoothing": 10.0,
        "use_relative_e_distances": false
    }))
    .unwrap();

    assert_ne!(
        layer_print_moves(&smoothed, 1),
        layer_print_moves(&baseline, 1)
    );
}

#[test]
fn spiral_mode_smooth_uses_original_second_layer_points_for_third_layer() {
    let baseline = spiral_gcode(json!({ "spiral_mode_smooth": false })).unwrap();
    let smoothed = spiral_gcode(json!({
        "spiral_mode_smooth": true,
        "spiral_mode_max_xy_smoothing": 10.0
    }))
    .unwrap();

    let second_original = layer_print_moves(&baseline, 1);
    let third_original = layer_print_moves(&baseline, 2);
    let second_smoothed = layer_print_moves(&smoothed, 1);
    let third_smoothed = layer_print_moves(&smoothed, 2);

    assert_eq!(second_original.len(), third_smoothed.len());
    assert_eq!(third_original.len(), third_smoothed.len());

    let expected_from_original = second_original[0].x * 0.75 + third_original[0].x * 0.25;
    let expected_from_smoothed = second_smoothed[0].x * 0.75 + third_original[0].x * 0.25;
    assert_close(third_smoothed[0].x, expected_from_original);
    assert!((third_smoothed[0].x - expected_from_smoothed).abs() > 0.05);
}

#[test]
fn spiral_mode_max_xy_smoothing_parses_percent_default_and_rejects_invalid_values() {
    let default_smoothed = spiral_gcode(json!({ "spiral_mode_smooth": true })).unwrap();
    let explicit_percent = spiral_gcode(json!({
        "spiral_mode_smooth": true,
        "spiral_mode_max_xy_smoothing": "200%"
    }))
    .unwrap();

    assert_eq!(
        layer_print_moves(&explicit_percent, 1),
        layer_print_moves(&default_smoothed, 1)
    );

    for value in [json!(-0.01), json!(1000.01), json!("NaN"), json!("bad%")] {
        let err = spiral_gcode(json!({
            "spiral_mode_smooth": true,
            "spiral_mode_max_xy_smoothing": value
        }))
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("spiral_mode_max_xy_smoothing"));
    }
}

fn spiral_gcode(extra: serde_json::Value) -> Result<String, SliceError> {
    let mut options = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 1,
        "line_width": 0.4,
        "nozzle_diameter": [0.4],
        "sparse_infill_density": 0,
        "bottom_shell_layers": 1,
        "top_shell_layers": 0,
        "spiral_mode": true,
        "seam_gap": 0,
        "gcode_comments": false
    });
    for (key, value) in extra.as_object().unwrap() {
        options[key] = value.clone();
    }
    let mut options: SliceOptions = serde_json::from_value(options).unwrap();
    options.normalize_fdm(0)?;
    let pipeline = contour_layers_pipeline_from_layers_for_tests(
        &options,
        vec![square(0.0), square(0.4), square(0.8), square(1.2)],
    );
    let output = format_gcode(&pipeline, &options)?;
    Ok(String::from_utf8(output).unwrap())
}

fn square(x_offset: f64) -> Vec<Contour> {
    vec![Contour::new(vec![
        Point2::new(x_offset, 0.0),
        Point2::new(x_offset + 4.0, 0.0),
        Point2::new(x_offset + 4.0, 4.0),
        Point2::new(x_offset, 4.0),
    ])]
}

fn layer_print_moves(output: &str, layer: usize) -> Vec<PrintedMove> {
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
            (in_layer && line.starts_with("G1 X") && line.contains(" E")).then(|| {
                let x = axis_value(line, 'X').unwrap();
                let y = axis_value(line, 'Y').unwrap();
                let e = axis_value(line, 'E').unwrap();
                PrintedMove { x, y, e }
            })
        })
        .collect()
}

fn axis_value(line: &str, axis: char) -> Option<f64> {
    line.split_whitespace()
        .find_map(|word| word.strip_prefix(axis))
        .and_then(|value| value.parse().ok())
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct PrintedMove {
    x: f64,
    y: f64,
    e: f64,
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 0.001,
        "actual {actual} expected {expected}"
    );
}
