use crate::{
    SliceOptions, gcode::format_gcode, pipeline::test_support::rectangular_layers_pipeline,
};
use serde_json::json;

#[test]
fn alternate_extra_wall_adds_odd_layer_internal_wall_gcode() {
    let options = options_with_sparse_density(20);

    let pipeline = rectangular_layers_pipeline(&options, 3);
    assert_eq!(pipeline.layer_perimeters()[0].paths().len(), 2);
    assert_eq!(pipeline.layer_perimeters()[1].paths().len(), 3);
    assert_eq!(pipeline.layer_perimeters()[2].paths().len(), 2);

    let gcode = String::from_utf8(format_gcode(&pipeline, &options).unwrap()).unwrap();
    let first_layer = layer_block(&gcode, 0);
    let second_layer = layer_block(&gcode, 1);
    let third_layer = layer_block(&gcode, 2);

    assert!(!first_layer.contains(";PERIMETER:internal:0.75708,0.75708 -> 3.24292,0.75708"));
    assert!(second_layer.contains(";PERIMETER:internal:0.75708,0.75708 -> 3.24292,0.75708"));
    assert!(
        second_layer.contains(";PRINT_PATH:internal_perimeter:0.75708,0.75708 -> 3.24292,0.75708")
    );
    assert!(!third_layer.contains(";PERIMETER:internal:0.75708,0.75708 -> 3.24292,0.75708"));
}

#[test]
fn alternate_extra_wall_requires_positive_sparse_infill_density() {
    let options = options_with_sparse_density(0);

    let pipeline = rectangular_layers_pipeline(&options, 2);
    assert_eq!(pipeline.layer_perimeters()[0].paths().len(), 2);
    assert_eq!(pipeline.layer_perimeters()[1].paths().len(), 2);

    let gcode = String::from_utf8(format_gcode(&pipeline, &options).unwrap()).unwrap();
    let second_layer = layer_block(&gcode, 1);

    assert!(!second_layer.contains(";PERIMETER:internal:0.75708,0.75708 -> 3.24292,0.75708"));
    assert!(
        !second_layer.contains(";PRINT_PATH:internal_perimeter:0.75708,0.75708 -> 3.24292,0.75708")
    );
}

fn options_with_sparse_density(sparse_infill_density: u32) -> SliceOptions {
    serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 2,
        "line_width": 0.4,
        "sparse_infill_density": sparse_infill_density,
        "alternate_extra_wall": true
    }))
    .unwrap()
}

fn layer_block(gcode: &str, layer_id: usize) -> &str {
    let marker = format!(";LAYER:{layer_id}\n");
    let start = gcode.find(&marker).expect("layer marker");
    let rest = &gcode[start..];
    let next = rest.find("\n;LAYER_CHANGE\n;LAYER:").unwrap_or(rest.len());
    &rest[..next]
}
