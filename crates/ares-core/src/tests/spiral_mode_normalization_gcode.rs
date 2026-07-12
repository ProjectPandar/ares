use super::*;
use serde_json::json;

#[tokio::test]
async fn slice_consumes_spiral_mode_normalization_for_gcode_paths() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 3,
        "line_width": 0.4,
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.25,
        "minimum_sparse_infill_area": 0,
        "infill_direction": 0,
        "bottom_shell_layers": 2,
        "top_shell_layers": 3,
        "spiral_mode": true
    }))
    .unwrap();

    let output = slice(square_pyramid_ascii_stl(), options).await.unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(output.contains("; total_perimeter_count = 2"));
    assert!(!output.contains("; total_infill_count = 0"));
    assert!(output.contains(";PRINT_PATH:external_perimeter:"));
    assert!(output.contains(";PRINT_PATH:bottom_surface:"));
    assert!(output.contains(";PRINT_PATH:top_solid_infill:"));
    assert!(output.contains(";EXTRUSION:print:external_perimeter:"));
    assert!(output.contains(";EXTRUSION:print:bottom_surface:"));
    assert!(output.contains(";EXTRUSION:print:top_solid_infill:"));
    assert!(!output.contains(";PRINT_PATH:internal_perimeter:"));
    assert!(!output.contains(";PRINT_PATH:sparse_infill:"));
    assert!(!output.contains(";EXTRUSION:print:sparse_infill:"));
}

#[tokio::test]
async fn slice_reports_invalid_spiral_mode_from_normalization() {
    let options: SliceOptions = serde_json::from_value(json!({
        "spiral_mode": "true"
    }))
    .unwrap();

    let err = slice(square_pyramid_ascii_stl(), options)
        .await
        .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("spiral_mode"));
}

#[tokio::test]
async fn slice_reports_invalid_spiral_retraction_arrays_from_normalization() {
    let options: SliceOptions = serde_json::from_value(json!({
        "spiral_mode": true,
        "retract_when_changing_layer": [true, "false"]
    }))
    .unwrap();

    let err = slice(square_pyramid_ascii_stl(), options)
        .await
        .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("retract_when_changing_layer"));
}
