use super::*;
use serde_json::json;

#[test]
fn extra_perimeters_on_overhangs_reaches_print_paths_moves_and_gcode_comments() {
    let off = options(json!({ "extra_perimeters_on_overhangs": false }));
    let on = options(json!({ "extra_perimeters_on_overhangs": true }));
    let off_pipeline = crate::pipeline::test_support::unsupported_second_layer_pipeline(&off);
    let on_pipeline = crate::pipeline::test_support::unsupported_second_layer_pipeline(&on);
    let on_gcode =
        String::from_utf8(crate::gcode::format_gcode(&on_pipeline, &on).unwrap()).unwrap();

    assert_eq!(layer_overhang_print_paths(&off_pipeline, 1), 1);
    assert_eq!(layer_overhang_print_paths(&on_pipeline, 1), 2);
    assert_eq!(layer_overhang_moves(&off_pipeline, 1), 4);
    assert_eq!(layer_overhang_moves(&on_pipeline, 1), 8);
    assert!(on_gcode.contains(";PERIMETER:overhang:10.4,0.4 -> 13.6,0.4 -> 13.6,3.6 -> 10.4,3.6"));
    assert!(
        on_gcode.contains(
            ";PRINT_PATH:overhang_perimeter:10.4,0.4 -> 13.6,0.4 -> 13.6,3.6 -> 10.4,3.6"
        )
    );
    assert!(on_gcode.contains(";MOVE:print:overhang_perimeter:13.6,0.4"));
}

fn options(extra: serde_json::Value) -> SliceOptions {
    let mut value = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 1,
        "line_width": 0.4,
        "sparse_infill_density": 0,
        "skirt_loops": 0,
        "brim_width": 0.0,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    });
    for (key, value_extra) in extra.as_object().unwrap() {
        value[key] = value_extra.clone();
    }
    serde_json::from_value(value).unwrap()
}

fn layer_overhang_print_paths(pipeline: &SlicingPipeline, layer_id: usize) -> usize {
    pipeline.layer_print_paths()[layer_id]
        .paths()
        .iter()
        .filter(|path| path.role() == PrintPathRole::OverhangPerimeter)
        .count()
}

fn layer_overhang_moves(pipeline: &SlicingPipeline, layer_id: usize) -> usize {
    pipeline.layer_toolpath_moves()[layer_id]
        .moves()
        .iter()
        .filter(|mov| {
            mov.kind() == crate::ToolpathMoveKind::Print
                && mov.role() == PrintPathRole::OverhangPerimeter
        })
        .count()
}
