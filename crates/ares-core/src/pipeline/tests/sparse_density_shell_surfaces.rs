use super::*;
use crate::{InfillRole, PrintPathRole};
use serde_json::json;

#[test]
fn sparse_density_shell_layers_reach_print_paths_and_gcode_as_solid_surfaces() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.5,
        "minimum_sparse_infill_area": 0,
        "sparse_infill_pattern": "rectilinear",
        "infill_direction": 0,
        "solid_infill_direction": 0,
        "wall_loops": 0,
        "skirt_loops": 0,
        "brim_width": 0,
        "infill_anchor_max": 0,
        "bottom_shell_layers": 1,
        "bottom_shell_thickness": 0,
        "top_shell_layers": 1,
        "top_shell_thickness": 0
    }))
    .unwrap();

    let pipeline = crate::pipeline::test_support::rectangular_layers_pipeline(&options, 3);
    let gcode =
        String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap();

    assert!(
        pipeline.layer_infills()[0]
            .paths()
            .iter()
            .all(|path| path.role() == InfillRole::Solid)
    );
    assert!(
        pipeline.layer_infills()[1]
            .paths()
            .iter()
            .all(|path| path.role() == InfillRole::Sparse)
    );
    assert!(
        pipeline.layer_infills()[2]
            .paths()
            .iter()
            .all(|path| path.role() == InfillRole::Solid)
    );
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
            .any(|path| path.role() == PrintPathRole::SparseInfill)
    );
    assert!(
        pipeline.layer_print_paths()[2]
            .paths()
            .iter()
            .any(|path| path.role() == PrintPathRole::TopSolidInfill)
    );
    assert!(gcode.contains(";PRINT_PATH:bottom_surface:"));
    assert!(gcode.contains(";PRINT_PATH:sparse_infill:"));
    assert!(gcode.contains(";PRINT_PATH:top_solid_infill:"));
}
