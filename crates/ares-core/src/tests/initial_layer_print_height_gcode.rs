use super::*;

#[tokio::test]
async fn initial_layer_print_height_controls_header_layer_z_and_placeholder() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_print_height": 0.32,
        "machine_start_gcode": ";FLH [first_layer_height]",
        "seam_gap": 0,
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.25,
        "minimum_sparse_infill_area": 0,
        "infill_direction": 0,
        "is_infill_first": true,
        "infill_anchor_max": 0,
        "bottom_shell_layers": 0,
        "top_shell_layers": 0
    }))
    .unwrap();

    let output = slice(square_pyramid_ascii_stl(), options).await.unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(
        output
            .lines()
            .any(|line| line == "; initial_layer_height = 0.32")
    );
    assert!(output.lines().any(|line| line == ";FLH 0.32"));
    assert!(output.contains(";LAYER_CHANGE\n;LAYER:0\n;Z:0.32\nG1 Z0.32"));
}

#[tokio::test]
async fn numeric_string_initial_layer_print_height_controls_layer_z() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_print_height": "0.24",
        "machine_start_gcode": ";FLH [first_layer_height]",
        "sparse_infill_density": 0
    }))
    .unwrap();

    let output = slice(square_pyramid_ascii_stl(), options).await.unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(
        output
            .lines()
            .any(|line| line == "; initial_layer_height = 0.24")
    );
    assert!(output.lines().any(|line| line == ";FLH 0.24"));
    assert!(output.contains(";LAYER_CHANGE\n;LAYER:0\n;Z:0.24\nG1 Z0.24"));
}

#[tokio::test]
async fn invalid_initial_layer_print_height_reaches_runtime_error() {
    for value in [
        json!(0.0),
        json!(-0.1),
        json!("NaN"),
        json!("inf"),
        json!("Infinity"),
        json!(true),
        json!([]),
        json!({ "height": 0.2 }),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "initial_layer_print_height": value
        }))
        .unwrap();

        let err = slice(square_pyramid_ascii_stl(), options)
            .await
            .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("initial_layer_print_height"));
    }
}
