use crate::{
    SliceOptions, gcode::format_gcode, pipeline::test_support::rectangular_layers_pipeline,
};
use serde_json::json;

#[test]
fn only_one_wall_top_removes_topmost_internal_wall_gcode() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 3,
        "line_width": 0.4,
        "sparse_infill_density": 0,
        "only_one_wall_top": true
    }))
    .unwrap();

    let pipeline = rectangular_layers_pipeline(&options, 3);
    assert_eq!(pipeline.layer_perimeters()[0].paths().len(), 3);
    assert_eq!(pipeline.layer_perimeters()[1].paths().len(), 3);
    assert_eq!(pipeline.layer_perimeters()[2].paths().len(), 1);

    let gcode = String::from_utf8(format_gcode(&pipeline, &options).unwrap()).unwrap();
    let middle_layer = layer_block(&gcode, 1);
    let top_layer = layer_block(&gcode, 2);

    assert!(middle_layer.contains(";PERIMETER:internal:0.75708,0.75708 -> 3.24292,0.75708"));
    assert!(
        middle_layer.contains(";PRINT_PATH:internal_perimeter:0.75708,0.75708 -> 3.24292,0.75708")
    );
    assert!(top_layer.contains(";PERIMETER:external:0,0 -> 4,0"));
    assert!(!top_layer.contains(";PERIMETER:internal:"));
    assert!(!top_layer.contains(";PRINT_PATH:internal_perimeter:"));
}

fn layer_block(gcode: &str, layer_id: usize) -> &str {
    let marker = format!(";LAYER:{layer_id}\n");
    let start = gcode.find(&marker).expect("layer marker");
    let rest = &gcode[start..];
    let next = rest.find("\n;LAYER_CHANGE\n;LAYER:").unwrap_or(rest.len());
    &rest[..next]
}
