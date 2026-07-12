use super::*;
use crate::{Point2, PrintPathRole};
use serde_json::json;

#[test]
fn bottom_surface_pattern_changes_bottom_surface_print_paths_and_gcode() {
    let base = solid_surface_options(json!({
        "bottom_surface_pattern": "rectilinear",
        "internal_solid_infill_pattern": "rectilinear"
    }));
    let aligned = solid_surface_options(json!({
        "bottom_surface_pattern": "alignedrectilinear",
        "internal_solid_infill_pattern": "rectilinear"
    }));

    let base_pipeline = crate::pipeline::test_support::rectangular_layers_pipeline(&base, 3);
    let aligned_pipeline = crate::pipeline::test_support::rectangular_layers_pipeline(&aligned, 3);
    let base_gcode =
        String::from_utf8(crate::gcode::format_gcode(&base_pipeline, &base).unwrap()).unwrap();
    let aligned_gcode =
        String::from_utf8(crate::gcode::format_gcode(&aligned_pipeline, &aligned).unwrap())
            .unwrap();

    assert!(
        base_pipeline.layer_print_paths()[1]
            .paths()
            .iter()
            .any(|path| {
                path.role() == PrintPathRole::BottomSurface
                    && path.points() == [Point2::new(4.0, 0.25), Point2::new(-0.0, 0.25)]
            })
    );
    assert!(
        aligned_pipeline.layer_print_paths()[1]
            .paths()
            .iter()
            .any(|path| {
                path.role() == PrintPathRole::BottomSurface
                    && path.points() == [Point2::new(0.25, 0.0), Point2::new(0.25, 4.0)]
            })
    );
    assert_ne!(
        layer_role_comment(&base_gcode, 1, "bottom_surface"),
        layer_role_comment(&aligned_gcode, 1, "bottom_surface")
    );
    assert!(base_gcode.contains(";PRINT_PATH:bottom_surface:4,0.25 -> 0,0.25"));
    assert!(aligned_gcode.contains(";PRINT_PATH:bottom_surface:0.25,0 -> 0.25,4"));
}

#[test]
fn top_surface_pattern_changes_top_surface_print_paths_and_gcode() {
    let base = solid_surface_options(json!({
        "bottom_shell_layers": 0,
        "top_surface_pattern": "rectilinear",
        "internal_solid_infill_pattern": "rectilinear"
    }));
    let aligned = solid_surface_options(json!({
        "bottom_shell_layers": 0,
        "top_surface_pattern": "alignedrectilinear",
        "internal_solid_infill_pattern": "rectilinear"
    }));

    let base_pipeline = crate::pipeline::test_support::rectangular_layers_pipeline(&base, 2);
    let aligned_pipeline = crate::pipeline::test_support::rectangular_layers_pipeline(&aligned, 2);
    let base_gcode =
        String::from_utf8(crate::gcode::format_gcode(&base_pipeline, &base).unwrap()).unwrap();
    let aligned_gcode =
        String::from_utf8(crate::gcode::format_gcode(&aligned_pipeline, &aligned).unwrap())
            .unwrap();

    assert!(
        base_pipeline.layer_print_paths()[1]
            .paths()
            .iter()
            .any(|path| {
                path.role() == PrintPathRole::TopSolidInfill
                    && path.points() == [Point2::new(4.0, 0.25), Point2::new(-0.0, 0.25)]
            })
    );
    assert!(
        aligned_pipeline.layer_print_paths()[1]
            .paths()
            .iter()
            .any(|path| {
                path.role() == PrintPathRole::TopSolidInfill
                    && path.points() == [Point2::new(0.25, 0.0), Point2::new(0.25, 4.0)]
            })
    );
    assert_ne!(
        layer_role_comment(&base_gcode, 1, "top_solid_infill"),
        layer_role_comment(&aligned_gcode, 1, "top_solid_infill")
    );
    assert!(base_gcode.contains(";PRINT_PATH:top_solid_infill:4,0.25 -> 0,0.25"));
    assert!(aligned_gcode.contains(";PRINT_PATH:top_solid_infill:0.25,0 -> 0.25,4"));
}

#[test]
fn bottom_surface_concentric_pattern_reaches_print_paths_and_gcode() {
    let options = solid_surface_options(json!({
        "bottom_surface_pattern": "concentric",
        "internal_solid_infill_pattern": "rectilinear"
    }));

    let pipeline = crate::pipeline::test_support::rectangular_layers_pipeline(&options, 3);
    let gcode =
        String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap();

    assert!(pipeline.layer_print_paths()[1].paths().iter().any(|path| {
        path.role() == PrintPathRole::BottomSurface
            && path.points() == [Point2::new(0.25, 0.25), Point2::new(3.75, 0.25)]
    }));
    assert!(gcode.contains(";PRINT_PATH:bottom_surface:0.25,0.25 -> 3.75,0.25"));
    assert!(gcode.contains(";PRINT_PATH:bottom_surface:3.75,0.25 -> 3.75,3.75"));
}

#[test]
fn top_surface_concentric_pattern_reaches_print_paths_and_gcode() {
    let options = solid_surface_options(json!({
        "bottom_shell_layers": 0,
        "top_surface_pattern": "concentric",
        "internal_solid_infill_pattern": "rectilinear"
    }));

    let pipeline = crate::pipeline::test_support::rectangular_layers_pipeline(&options, 2);
    let gcode =
        String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap();

    assert!(pipeline.layer_print_paths()[1].paths().iter().any(|path| {
        path.role() == PrintPathRole::TopSolidInfill
            && path.points() == [Point2::new(0.25, 0.25), Point2::new(3.75, 0.25)]
    }));
    assert!(gcode.contains(";PRINT_PATH:top_solid_infill:0.25,0.25 -> 3.75,0.25"));
    assert!(gcode.contains(";PRINT_PATH:top_solid_infill:3.75,0.25 -> 3.75,3.75"));
}

fn solid_surface_options(extra: serde_json::Value) -> SliceOptions {
    let mut value = json!({
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
        "bottom_shell_layers": 2,
        "top_shell_layers": 1
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

fn layer_role_comment<'a>(gcode: &'a str, layer_index: usize, role: &str) -> &'a str {
    let section = layer_section(gcode, layer_index);
    let prefix = format!(";PRINT_PATH:{role}:");
    section
        .lines()
        .find(|line| line.starts_with(&prefix))
        .unwrap()
}

fn layer_section(gcode: &str, layer_index: usize) -> &str {
    let marker = format!(";LAYER:{layer_index}");
    let start = gcode.find(&marker).unwrap();
    let rest = &gcode[start..];
    let next = format!(";LAYER:{}", layer_index + 1);
    rest.find(&next).map_or(rest, |end| &rest[..end])
}
