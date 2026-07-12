use super::*;
use crate::{InfillRole, PrintPathRole};
use serde_json::json;

#[test]
fn extra_solid_infills_reaches_print_paths_extrusion_speed_and_gcode() {
    let options = extra_solid_options(json!({ "extra_solid_infills": "2" }));
    let pipeline = crate::pipeline::test_support::rectangular_layers_pipeline(&options, 3);
    let gcode =
        String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap();

    assert!(
        pipeline.layer_infills()[0]
            .paths()
            .iter()
            .all(|path| path.role() == InfillRole::Sparse)
    );
    assert!(
        pipeline.layer_infills()[1]
            .paths()
            .iter()
            .all(|path| path.role() == InfillRole::Solid)
    );
    assert!(
        pipeline.layer_infills()[2]
            .paths()
            .iter()
            .all(|path| path.role() == InfillRole::Sparse)
    );
    assert!(
        pipeline.layer_print_paths()[1]
            .paths()
            .iter()
            .any(|path| path.role() == PrintPathRole::SolidInfill)
    );
    assert!(
        pipeline.layer_toolpath_moves()[1]
            .moves()
            .iter()
            .any(|move_| move_.role() == PrintPathRole::SolidInfill)
    );
    assert!(
        pipeline.layer_extrusion_moves()[1]
            .moves()
            .iter()
            .any(|move_| move_.role() == PrintPathRole::SolidInfill)
    );
    assert!(
        pipeline.layer_speed_moves()[1]
            .moves()
            .iter()
            .any(|move_| move_.role() == PrintPathRole::SolidInfill)
    );
    assert!(gcode.contains(";PRINT_PATH:solid_infill:"));
    assert!(gcode.contains(";EXTRUSION:print:solid_infill:"));
    assert!(gcode.contains(";SPEED:print:solid_infill:"));
}

#[test]
fn extra_solid_infills_preserves_shell_print_path_roles() {
    let options = extra_solid_options(json!({
        "extra_solid_infills": "1#",
        "bottom_shell_layers": 1,
        "top_shell_layers": 1
    }));
    let pipeline = crate::pipeline::test_support::rectangular_layers_pipeline(&options, 3);

    assert!(
        pipeline.layer_print_paths()[0]
            .paths()
            .iter()
            .any(|path| path.role() == PrintPathRole::BottomSurface)
    );
    assert!(
        pipeline.layer_print_paths()[1]
            .paths()
            .iter()
            .any(|path| path.role() == PrintPathRole::SolidInfill)
    );
    assert!(
        pipeline.layer_print_paths()[2]
            .paths()
            .iter()
            .any(|path| path.role() == PrintPathRole::TopSolidInfill)
    );
}

#[test]
fn extra_solid_infills_does_not_create_infill_when_sparse_density_is_zero() {
    let options = extra_solid_options(json!({
        "extra_solid_infills": "1#",
        "sparse_infill_density": 0
    }));
    let pipeline = crate::pipeline::test_support::rectangular_layers_pipeline(&options, 3);

    assert!(
        pipeline
            .layer_infills()
            .iter()
            .all(|layer| layer.paths().is_empty())
    );
    assert!(
        !pipeline
            .layer_print_paths()
            .iter()
            .flat_map(|layer| layer.paths())
            .any(|path| path.role() == PrintPathRole::SolidInfill)
    );
}

fn extra_solid_options(overrides: serde_json::Value) -> SliceOptions {
    let mut value = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.5,
        "minimum_sparse_infill_area": 0,
        "sparse_infill_pattern": "rectilinear",
        "internal_solid_infill_pattern": "grid",
        "solid_infill_direction": 0,
        "wall_loops": 0,
        "skirt_loops": 0,
        "brim_width": 0,
        "infill_anchor_max": 0,
        "bottom_shell_layers": 0,
        "bottom_shell_thickness": 0,
        "top_shell_layers": 0,
        "top_shell_thickness": 0
    });
    value.as_object_mut().unwrap().extend(
        overrides
            .as_object()
            .unwrap()
            .iter()
            .map(|(key, value)| (key.clone(), value.clone())),
    );
    serde_json::from_value(value).unwrap()
}
