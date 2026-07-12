use crate::{
    SliceOptions, gcode::format_gcode, pipeline::test_support::rectangular_layers_pipeline,
};
use serde_json::json;

#[test]
fn legacy_wall_infill_order_infill_first_changes_non_first_layer_gcode_order() {
    let options: SliceOptions = serde_json::from_value(json!({
        "wall_infill_order": "infill/inner wall/outer wall",
        "wall_loops": 2,
        "line_width": 0.4,
        "sparse_infill_density": 50,
        "minimum_sparse_infill_area": 0,
        "bottom_shell_layers": 0,
        "top_shell_layers": 0,
        "infill_direction": 0
    }))
    .unwrap();

    let pipeline = rectangular_layers_pipeline(&options, 2);
    let gcode = String::from_utf8(format_gcode(&pipeline, &options).unwrap()).unwrap();

    let layer_zero = layer_block(&gcode, 0);
    let layer_one = layer_block(&gcode, 1);

    assert!(
        marker_position(layer_zero, ";PRINT_PATH:external_perimeter:")
            < marker_position(layer_zero, ";PRINT_PATH:sparse_infill:")
    );
    assert!(
        marker_position(layer_one, ";PRINT_PATH:sparse_infill:")
            < marker_position(layer_one, ";PRINT_PATH:internal_perimeter:")
    );
}

fn layer_block(gcode: &str, layer_id: usize) -> &str {
    let marker = format!(";LAYER:{layer_id}\n");
    let start = gcode.find(&marker).expect("layer marker") + marker.len();
    let rest = &gcode[start..];
    let end = rest.find("\n;LAYER_CHANGE\n;LAYER:").unwrap_or(rest.len());
    &rest[..end]
}

fn marker_position(layer: &str, marker: &str) -> usize {
    layer.find(marker).expect(marker)
}
