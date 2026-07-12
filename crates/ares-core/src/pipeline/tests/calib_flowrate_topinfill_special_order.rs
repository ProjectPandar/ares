use super::*;
use serde_json::json;

#[test]
fn calibration_reverses_top_surface_through_pipeline_and_gcode() {
    let disabled = calibration_options(false);
    let enabled = calibration_options(true);
    let disabled_pipeline =
        crate::pipeline::test_support::rectangular_layers_pipeline(&disabled, 3);
    let enabled_pipeline = crate::pipeline::test_support::rectangular_layers_pipeline(&enabled, 3);
    let disabled_gcode =
        String::from_utf8(crate::gcode::format_gcode(&disabled_pipeline, &disabled).unwrap())
            .unwrap();
    let enabled_gcode =
        String::from_utf8(crate::gcode::format_gcode(&enabled_pipeline, &enabled).unwrap())
            .unwrap();

    assert_ne!(
        top_infill_segments(&disabled_pipeline.layer_infills()[2]),
        top_infill_segments(&enabled_pipeline.layer_infills()[2])
    );
    assert_eq!(
        top_infill_segments(&enabled_pipeline.layer_infills()[2]),
        reversed_segments(&top_infill_segments(&disabled_pipeline.layer_infills()[2]))
    );
    assert_ne!(
        top_print_path_segments(&disabled_pipeline.layer_print_paths()[2]),
        top_print_path_segments(&enabled_pipeline.layer_print_paths()[2])
    );
    assert_eq!(
        top_print_path_segments(&enabled_pipeline.layer_print_paths()[2]),
        reversed_segments(&top_print_path_segments(
            &disabled_pipeline.layer_print_paths()[2]
        ))
    );
    assert!(disabled_gcode.contains(";INFILL:solid:0.25,0 -> 0.25,4"));
    assert!(enabled_gcode.contains(";INFILL:solid:0.25,4 -> 0.25,0"));
    assert!(disabled_gcode.contains(";PRINT_PATH:top_solid_infill:0.25,0 -> 0.25,4"));
    assert!(enabled_gcode.contains(";PRINT_PATH:top_solid_infill:0.25,4 -> 0.25,0"));
    assert!(enabled_gcode.contains(";EXTRUSION:print:top_solid_infill:"));
}

fn calibration_options(enabled: bool) -> SliceOptions {
    serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 100,
        "line_width": 0.5,
        "sparse_infill_line_width": 0.5,
        "minimum_sparse_infill_area": 0,
        "solid_infill_direction": 0,
        "wall_loops": 0,
        "skirt_loops": 0,
        "brim_width": 0,
        "infill_anchor_max": 0,
        "bottom_shell_layers": 1,
        "bottom_shell_thickness": 0,
        "top_shell_layers": 1,
        "top_shell_thickness": 0,
        "top_surface_pattern": "alignedrectilinear",
        "calib_flowrate_topinfill_special_order": enabled
    }))
    .unwrap()
}

fn top_infill_segments(layer: &LayerInfills) -> Vec<Vec<Point2>> {
    layer
        .paths()
        .iter()
        .map(|path| path.points().to_vec())
        .collect()
}

fn top_print_path_segments(layer: &LayerPrintPaths) -> Vec<Vec<Point2>> {
    layer
        .paths()
        .iter()
        .filter(|path| path.role() == PrintPathRole::TopSolidInfill)
        .map(|path| path.points().to_vec())
        .collect()
}

fn reversed_segments(segments: &[Vec<Point2>]) -> Vec<Vec<Point2>> {
    segments
        .iter()
        .map(|segment| vec![segment[1], segment[0]])
        .collect()
}
