use super::*;
use crate::pipeline::test_support::{
    rectangular_layers_pipeline, rectangular_pipeline, single_path_pipeline,
};

fn first_role_delta(pipeline: &SlicingPipeline, role: PrintPathRole) -> f64 {
    let mut previous = 0.0;
    for layer in pipeline.layer_extrusion_moves() {
        for movement in layer.moves() {
            let Some(e) = movement.e_position() else {
                continue;
            };
            if movement.role() == role {
                return e - previous;
            }
            previous = e;
        }
    }
    panic!("missing role {role:?}");
}

fn first_role_width(pipeline: &SlicingPipeline, role: PrintPathRole) -> f64 {
    pipeline
        .layer_extrusion_moves()
        .iter()
        .flat_map(|layer| layer.moves())
        .find(|movement| movement.role() == role && movement.e_position().is_some())
        .and_then(|movement| movement.effective_line_width_mm())
        .unwrap()
}

fn assert_approx_eq(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 0.000001,
        "actual {actual} expected {expected}"
    );
}

#[test]
fn wall_filament_changes_perimeter_extrusion_and_auto_width() {
    let first: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": [0.4, 0.8],
        "filament_diameter": [1.75, 2.85],
        "wall_filament": 1,
        "wall_loops": 2,
        "line_width": 0,
        "outer_wall_line_width": 0,
        "inner_wall_line_width": 0,
        "sparse_infill_density": 0,
        "skirt_loops": 0,
        "brim_width": 0
    }))
    .unwrap();
    let second: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": [0.4, 0.8],
        "filament_diameter": [1.75, 2.85],
        "wall_filament": "2",
        "wall_loops": 2,
        "line_width": 0,
        "outer_wall_line_width": 0,
        "inner_wall_line_width": 0,
        "sparse_infill_density": 0,
        "skirt_loops": 0,
        "brim_width": 0
    }))
    .unwrap();

    let first_pipeline = rectangular_pipeline(&first);
    let second_pipeline = rectangular_pipeline(&second);

    assert_approx_eq(
        first_role_width(&first_pipeline, PrintPathRole::ExternalPerimeter),
        0.45,
    );
    assert_approx_eq(
        first_role_width(&second_pipeline, PrintPathRole::ExternalPerimeter),
        0.9,
    );
    assert_ne!(
        first_role_delta(&first_pipeline, PrintPathRole::ExternalPerimeter),
        first_role_delta(&second_pipeline, PrintPathRole::ExternalPerimeter)
    );
}

#[test]
fn sparse_infill_filament_changes_sparse_extrusion_without_moving_wall_selector() {
    let first: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": [0.4, 0.8],
        "filament_diameter": [1.75, 2.85],
        "wall_filament": 1,
        "sparse_infill_filament": 1,
        "wall_loops": 1,
        "line_width": 0,
        "outer_wall_line_width": 0,
        "sparse_infill_line_width": 0,
        "sparse_infill_density": 50,
        "minimum_sparse_infill_area": 0,
        "infill_direction": 0,
        "skirt_loops": 0,
        "brim_width": 0,
        "bottom_shell_layers": 0,
        "top_shell_layers": 0,
        "infill_anchor_max": 0
    }))
    .unwrap();
    let second: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": [0.4, 0.8],
        "filament_diameter": [1.75, 2.85],
        "wall_filament": 1,
        "sparse_infill_filament": 2,
        "wall_loops": 1,
        "line_width": 0,
        "outer_wall_line_width": 0,
        "sparse_infill_line_width": 0,
        "sparse_infill_density": 50,
        "minimum_sparse_infill_area": 0,
        "infill_direction": 0,
        "skirt_loops": 0,
        "brim_width": 0,
        "bottom_shell_layers": 0,
        "top_shell_layers": 0,
        "infill_anchor_max": 0
    }))
    .unwrap();

    let first_pipeline = rectangular_pipeline(&first);
    let second_pipeline = rectangular_pipeline(&second);

    assert_approx_eq(
        first_role_width(&second_pipeline, PrintPathRole::ExternalPerimeter),
        0.45,
    );
    assert_approx_eq(
        first_role_width(&second_pipeline, PrintPathRole::SparseInfill),
        0.9,
    );
    assert_ne!(
        first_role_delta(&first_pipeline, PrintPathRole::SparseInfill),
        first_role_delta(&second_pipeline, PrintPathRole::SparseInfill)
    );
}

#[test]
fn solid_infill_filament_changes_solid_top_and_bottom_surface_extrusion() {
    let first: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": [0.4, 0.8],
        "filament_diameter": [1.75, 2.85],
        "solid_infill_filament": 1,
        "sparse_infill_density": 100,
        "minimum_sparse_infill_area": 0,
        "wall_loops": 0,
        "line_width": 0,
        "internal_solid_infill_line_width": 0,
        "top_surface_line_width": 0,
        "skirt_loops": 0,
        "brim_width": 0,
        "infill_anchor_max": 0,
        "bottom_shell_layers": 1,
        "bottom_shell_thickness": 0,
        "top_shell_layers": 1,
        "top_shell_thickness": 0
    }))
    .unwrap();
    let second: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": [0.4, 0.8],
        "filament_diameter": [1.75, 2.85],
        "solid_infill_filament": 2,
        "sparse_infill_density": 100,
        "minimum_sparse_infill_area": 0,
        "wall_loops": 0,
        "line_width": 0,
        "internal_solid_infill_line_width": 0,
        "top_surface_line_width": 0,
        "skirt_loops": 0,
        "brim_width": 0,
        "infill_anchor_max": 0,
        "bottom_shell_layers": 1,
        "bottom_shell_thickness": 0,
        "top_shell_layers": 1,
        "top_shell_thickness": 0
    }))
    .unwrap();

    let first_pipeline = rectangular_layers_pipeline(&first, 3);
    let second_pipeline = rectangular_layers_pipeline(&second, 3);

    assert_approx_eq(
        first_role_width(&first_pipeline, PrintPathRole::BottomSurface),
        0.45,
    );
    assert_approx_eq(
        first_role_width(&second_pipeline, PrintPathRole::BottomSurface),
        0.9,
    );
    assert_approx_eq(
        first_role_width(&first_pipeline, PrintPathRole::SolidInfill),
        0.45,
    );
    assert_approx_eq(
        first_role_width(&second_pipeline, PrintPathRole::SolidInfill),
        0.9,
    );
    assert_approx_eq(
        first_role_width(&first_pipeline, PrintPathRole::TopSolidInfill),
        0.4,
    );
    assert_approx_eq(
        first_role_width(&second_pipeline, PrintPathRole::TopSolidInfill),
        0.8,
    );

    for role in [
        PrintPathRole::BottomSurface,
        PrintPathRole::SolidInfill,
        PrintPathRole::TopSolidInfill,
    ] {
        assert_ne!(
            first_role_delta(&first_pipeline, role),
            first_role_delta(&second_pipeline, role)
        );
    }
}

#[test]
fn ironing_uses_solid_filament_selector_in_synthetic_pipeline() {
    let first: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": [0.4, 0.8],
        "filament_diameter": [1.75, 2.85],
        "solid_infill_filament": 1,
        "line_width": 0,
        "top_surface_line_width": 0
    }))
    .unwrap();
    let second: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": [0.4, 0.8],
        "filament_diameter": [1.75, 2.85],
        "solid_infill_filament": 2,
        "line_width": 0,
        "top_surface_line_width": 0
    }))
    .unwrap();

    let first_pipeline = single_path_pipeline(&first, PrintPathRole::Ironing, 0);
    let second_pipeline = single_path_pipeline(&second, PrintPathRole::Ironing, 0);

    assert_approx_eq(
        first_role_width(&first_pipeline, PrintPathRole::Ironing),
        0.4,
    );
    assert_approx_eq(
        first_role_width(&second_pipeline, PrintPathRole::Ironing),
        0.8,
    );
    assert_ne!(
        first_role_delta(&first_pipeline, PrintPathRole::Ironing),
        first_role_delta(&second_pipeline, PrintPathRole::Ironing)
    );
}
