use super::*;

#[tokio::test]
async fn rectilinear_sparse_infill_rotates_second_layer_gcode_paths() {
    let output = slice_infill_pattern_output("rectilinear").await;

    assert!(output.contains(";PRINT_PATH:sparse_infill:-0.25,-0.25 -> -0.25,0.25"));
    assert!(output.contains(";PRINT_PATH:sparse_infill:0.25,-0.75 -> -0.25,-0.75"));
}

#[tokio::test]
async fn aligned_rectilinear_sparse_infill_keeps_second_layer_gcode_paths_aligned() {
    let output = slice_infill_pattern_output("alignedrectilinear").await;

    assert!(output.contains(";PRINT_PATH:sparse_infill:-0.25,-0.25 -> -0.25,0.25"));
    assert!(output.contains(";PRINT_PATH:sparse_infill:-0.75,-0.25 -> -0.75,0.25"));
    assert!(!output.contains(";PRINT_PATH:sparse_infill:-1,-0.75 -> 0,-0.75"));
}

async fn slice_infill_pattern_output(pattern: &str) -> String {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.25,
        "minimum_sparse_infill_area": 0,
        "infill_direction": 0,
        "sparse_infill_pattern": pattern,
        "is_infill_first": true,
        "infill_anchor_max": 0,
        "bottom_shell_layers": 0,
        "top_shell_layers": 0
    }))
    .unwrap();
    let output = slice(square_pyramid_ascii_stl(), options).await.unwrap();
    String::from_utf8(output).unwrap()
}
