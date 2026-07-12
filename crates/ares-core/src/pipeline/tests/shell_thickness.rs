use super::*;
use crate::{ExtrusionRole, PrintPathRole};
use serde_json::json;

fn shell_options(extra: serde_json::Value) -> SliceOptions {
    let mut value = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "minimum_sparse_infill_area": 0,
        "wall_loops": 0,
        "skirt_loops": 0,
        "brim_width": 0,
        "infill_anchor_max": 0,
        "bottom_shell_layers": 1,
        "bottom_shell_thickness": 0.45,
        "top_shell_layers": 1,
        "top_shell_thickness": 0.45
    });
    value.as_object_mut().unwrap().extend(
        extra
            .as_object()
            .unwrap()
            .iter()
            .map(|(key, value)| (key.clone(), value.clone())),
    );
    serde_json::from_value(value).unwrap()
}

fn gcode_for(options: &SliceOptions, layer_count: usize) -> (crate::SlicingPipeline, String) {
    let pipeline = crate::pipeline::test_support::rectangular_layers_pipeline(options, layer_count);
    let gcode = String::from_utf8(crate::gcode::format_gcode(&pipeline, options).unwrap()).unwrap();
    (pipeline, gcode)
}

fn layer_block(gcode: &str, layer_id: usize) -> &str {
    let marker = format!(";LAYER:{layer_id}\n");
    let start = gcode.find(&marker).unwrap();
    let rest = &gcode[start + marker.len()..];
    let end = rest.find(";LAYER_CHANGE\n").unwrap_or(rest.len());
    &rest[..end]
}

fn print_path_has_role(
    pipeline: &crate::SlicingPipeline,
    layer_index: usize,
    role: PrintPathRole,
) -> bool {
    pipeline.layer_print_paths()[layer_index]
        .paths()
        .iter()
        .any(|path| path.role() == role)
}

fn fill_has_role(
    pipeline: &crate::SlicingPipeline,
    layer_index: usize,
    role: ExtrusionRole,
) -> bool {
    pipeline.print().objects()[0].layers()[layer_index].regions()[0]
        .fills()
        .paths()
        .iter()
        .any(|path| path.role() == role)
}

#[test]
fn bottom_shell_thickness_expands_dense_bottom_surface_gcode() {
    let options = shell_options(json!({ "sparse_infill_density": 100, "top_shell_layers": 0 }));
    let (pipeline, gcode) = gcode_for(&options, 4);

    assert!(print_path_has_role(
        &pipeline,
        2,
        PrintPathRole::BottomSurface
    ));
    assert!(fill_has_role(&pipeline, 2, ExtrusionRole::BottomSurface));
    assert!(layer_block(&gcode, 2).contains(";PRINT_PATH:bottom_surface:"));
}

#[test]
fn top_shell_thickness_expands_dense_top_surface_gcode() {
    let options = shell_options(json!({ "sparse_infill_density": 100, "bottom_shell_layers": 0 }));
    let (pipeline, gcode) = gcode_for(&options, 4);

    assert!(print_path_has_role(
        &pipeline,
        1,
        PrintPathRole::TopSolidInfill
    ));
    assert!(fill_has_role(&pipeline, 1, ExtrusionRole::TopSolidInfill));
    assert!(layer_block(&gcode, 1).contains(";PRINT_PATH:top_solid_infill:"));
}

#[test]
fn sparse_bottom_shell_thickness_expands_surface_roles() {
    let options = shell_options(json!({ "sparse_infill_density": 25, "top_shell_layers": 0 }));
    let (pipeline, _) = gcode_for(&options, 4);

    assert!(fill_has_role(&pipeline, 2, ExtrusionRole::BottomSurface));
}

#[test]
fn sparse_top_shell_thickness_expands_surface_roles() {
    let options = shell_options(json!({ "sparse_infill_density": 25, "bottom_shell_layers": 0 }));
    let (pipeline, _) = gcode_for(&options, 4);

    assert!(fill_has_role(&pipeline, 1, ExtrusionRole::TopSolidInfill));
}

#[test]
fn sparse_overlap_prefers_bottom_surface_role() {
    let options = shell_options(json!({ "sparse_infill_density": 25 }));
    let (pipeline, gcode) = gcode_for(&options, 4);

    assert!(print_path_has_role(
        &pipeline,
        1,
        PrintPathRole::BottomSurface
    ));
    assert!(!print_path_has_role(
        &pipeline,
        1,
        PrintPathRole::TopSolidInfill
    ));
    assert!(layer_block(&gcode, 1).contains(";PRINT_PATH:bottom_surface:"));
    assert!(!layer_block(&gcode, 1).contains(";PRINT_PATH:top_solid_infill:"));
}

#[test]
fn zero_sparse_density_remains_empty_with_shell_thickness() {
    let options = shell_options(json!({ "sparse_infill_density": 0 }));
    let pipeline = crate::pipeline::test_support::rectangular_layers_pipeline(&options, 4);

    assert!(
        pipeline
            .layer_infills()
            .iter()
            .all(|layer| layer.paths().is_empty())
    );
}
