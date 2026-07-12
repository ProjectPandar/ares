use super::*;
use crate::pipeline::test_support::rectangular_layers_pipeline;
use serde_json::json;

fn combination_options(enabled: bool) -> SliceOptions {
    serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "nozzle_diameter": [0.4],
        "wall_loops": 0,
        "skirt_loops": 0,
        "brim_width": 0,
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.5,
        "minimum_sparse_infill_area": 0,
        "sparse_infill_pattern": "rectilinear",
        "infill_direction": 0,
        "infill_combination": enabled,
        "infill_combination_max_layer_height": 0.4,
        "bottom_shell_layers": 0,
        "top_shell_layers": 0
    }))
    .unwrap()
}

fn layer_block(gcode: &str, layer_id: usize) -> &str {
    let marker = format!(";LAYER:{layer_id}\n");
    let start = gcode.find(&marker).unwrap();
    let rest = &gcode[start..];
    let next = rest[marker.len()..]
        .find(";LAYER_CHANGE\n")
        .map(|index| marker.len() + index)
        .unwrap_or(rest.len());
    &rest[..next]
}

#[test]
fn pipeline_combines_sparse_infill_into_upper_layer() {
    let options = combination_options(true);
    let pipeline = rectangular_layers_pipeline(&options, 3);

    assert_eq!(pipeline.layer_infills()[0].paths().len(), 4);
    assert!(pipeline.layer_infills()[1].paths().is_empty());
    assert_eq!(pipeline.layer_infills()[2].paths().len(), 4);
    assert!(
        pipeline.layer_print_paths()[1]
            .paths()
            .iter()
            .all(|path| path.role() != PrintPathRole::SparseInfill)
    );
    assert!(
        pipeline.layer_print_paths()[2]
            .paths()
            .iter()
            .any(|path| path.role() == PrintPathRole::SparseInfill)
    );
}

#[test]
fn combined_sparse_infill_extrudes_more_than_uncombined_target_layer() {
    let combined = rectangular_layers_pipeline(&combination_options(true), 3);
    let uncombined = rectangular_layers_pipeline(&combination_options(false), 3);

    assert!(
        combined.layer_extrusion_moves()[2].total_extrusion_mm()
            > uncombined.layer_extrusion_moves()[2].total_extrusion_mm()
    );
    assert_eq!(
        combined.layer_extrusion_moves()[1].total_extrusion_mm(),
        0.0
    );
}

#[test]
fn combined_sparse_infill_reaches_gcode_only_on_target_layer() {
    let options = combination_options(true);
    let pipeline = rectangular_layers_pipeline(&options, 3);
    let gcode =
        String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap();
    let lower = layer_block(&gcode, 1);
    let target = layer_block(&gcode, 2);

    assert!(lower.contains(";Z:0.4\n"));
    assert!(lower.contains("; infill_count = 0\n"));
    assert!(!lower.contains(";PRINT_PATH:sparse_infill:"));
    assert!(target.contains(";Z:0.6\n"));
    assert!(target.contains("; infill_count = 4\n"));
    assert!(target.contains(";PRINT_PATH:sparse_infill:"));
}
