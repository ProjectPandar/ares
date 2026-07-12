use super::*;
use crate::{SliceError, SliceOptions};
use serde_json::json;

fn assert_approx_eq(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 0.000001,
        "actual {actual} expected {expected}"
    );
}

fn base_options() -> ExtrusionOptions {
    ExtrusionOptions::new_for_tests(0.4, 1.75, 0.0, (0.0, 0.0), 0.0)
        .with_top_surface_line_width_spec(ExtrusionWidthSpec::auto())
        .with_internal_solid_infill_line_width_spec(ExtrusionWidthSpec::auto())
        .with_role_hardware_for_tests(
            RoleHardwareValues::new(0.4, 1.75),
            RoleHardwareValues::new(0.8, 2.85),
            RoleHardwareValues::new(0.8, 2.85),
        )
}

#[test]
fn role_nozzles_drive_automatic_widths() {
    let options = base_options();

    assert_approx_eq(
        options.width_for_role(PrintPathRole::ExternalPerimeter),
        0.45,
    );
    assert_approx_eq(
        options.width_for_role(PrintPathRole::InternalPerimeter),
        0.45,
    );
    assert_approx_eq(options.width_for_role(PrintPathRole::SparseInfill), 0.9);
    assert_approx_eq(options.width_for_role(PrintPathRole::SolidInfill), 0.9);
    assert_approx_eq(options.width_for_role(PrintPathRole::BottomSurface), 0.9);
    assert_approx_eq(options.width_for_role(PrintPathRole::TopSolidInfill), 0.8);
    assert_approx_eq(options.width_for_role(PrintPathRole::Ironing), 0.8);
}

#[test]
fn role_nozzles_drive_percentage_widths() {
    let options = ExtrusionOptions::new_for_tests(0.4, 1.75, 0.0, (0.0, 0.0), 0.0)
        .with_line_width_spec(ExtrusionWidthSpec::percent(120.0))
        .with_outer_wall_line_width_spec(ExtrusionWidthSpec::percent(110.0))
        .with_inner_wall_line_width_spec(ExtrusionWidthSpec::percent(115.0))
        .with_sparse_infill_line_width_spec(ExtrusionWidthSpec::percent(125.0))
        .with_internal_solid_infill_line_width_spec(ExtrusionWidthSpec::percent(130.0))
        .with_top_surface_line_width_spec(ExtrusionWidthSpec::percent(140.0))
        .with_role_hardware_for_tests(
            RoleHardwareValues::new(0.8, 2.85),
            RoleHardwareValues::new(0.8, 2.85),
            RoleHardwareValues::new(0.8, 2.85),
        );

    assert_approx_eq(
        options.width_for_role(PrintPathRole::ExternalPerimeter),
        0.88,
    );
    assert_approx_eq(
        options.width_for_role(PrintPathRole::InternalPerimeter),
        0.92,
    );
    assert_approx_eq(options.width_for_role(PrintPathRole::SparseInfill), 1.0);
    assert_approx_eq(options.width_for_role(PrintPathRole::SolidInfill), 1.04);
    assert_approx_eq(options.width_for_role(PrintPathRole::BottomSurface), 1.04);
    assert_approx_eq(options.width_for_role(PrintPathRole::TopSolidInfill), 1.12);
    assert_approx_eq(options.width_for_role(PrintPathRole::Ironing), 1.12);
}

#[test]
fn role_filament_diameter_changes_e_delta() {
    let base = ExtrusionOptions::new_for_tests(0.4, 1.75, 0.4, (0.4, 0.4), 0.4)
        .with_role_hardware_for_tests(
            RoleHardwareValues::new(0.4, 1.75),
            RoleHardwareValues::new(0.4, 1.75),
            RoleHardwareValues::new(0.4, 1.75),
        );
    let selected = base.with_role_hardware_for_tests(
        RoleHardwareValues::new(0.4, 2.85),
        RoleHardwareValues::new(0.4, 2.85),
        RoleHardwareValues::new(0.4, 2.85),
    );

    assert!(
        selected
            .extrusion_delta_for_segment(PrintPathRole::ExternalPerimeter, 0.2, false, 10.0)
            .unwrap()
            < base
                .extrusion_delta_for_segment(PrintPathRole::ExternalPerimeter, 0.2, false, 10.0)
                .unwrap()
    );
}

#[test]
fn out_of_range_role_hardware_falls_back_per_vector() {
    let options = ExtrusionOptions::new_for_tests(0.4, 1.75, 0.0, (0.0, 0.0), 0.0)
        .with_role_hardware_for_tests(
            RoleHardwareValues::new(0.4, 1.75),
            RoleHardwareValues::new(0.4, 2.85),
            RoleHardwareValues::new(0.4, 1.75),
        );

    assert_approx_eq(
        options.width_for_role(PrintPathRole::ExternalPerimeter),
        0.45,
    );
    assert_approx_eq(options.width_for_role(PrintPathRole::SparseInfill), 0.45);
    assert_approx_eq(options.width_for_role(PrintPathRole::SolidInfill), 0.45);
    assert!(
        options
            .extrusion_delta_for_segment(PrintPathRole::SparseInfill, 0.2, false, 10.0)
            .unwrap()
            < options
                .extrusion_delta_for_segment(PrintPathRole::ExternalPerimeter, 0.2, false, 10.0)
                .unwrap()
    );
}

#[test]
fn percent_line_width_fallback_uses_selected_role_nozzle() {
    let options = ExtrusionOptions::new_for_tests(0.4, 1.75, 0.0, (0.0, 0.0), 0.0)
        .with_line_width_spec(ExtrusionWidthSpec::percent(120.0))
        .with_role_hardware_for_tests(
            RoleHardwareValues::new(0.8, 2.85),
            RoleHardwareValues::new(0.8, 2.85),
            RoleHardwareValues::new(0.8, 2.85),
        );

    assert_approx_eq(
        options.width_for_role(PrintPathRole::ExternalPerimeter),
        0.96,
    );
    assert_approx_eq(options.width_for_role(PrintPathRole::SparseInfill), 0.96);
    assert_approx_eq(options.width_for_role(PrintPathRole::SolidInfill), 0.96);
}

#[test]
fn slice_options_missing_role_filaments_default_to_first_hardware() {
    let options: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": [0.4, 0.8],
        "filament_diameter": [1.75, 2.85],
        "line_width": 0,
        "outer_wall_line_width": 0,
        "sparse_infill_line_width": 0,
        "internal_solid_infill_line_width": 0
    }))
    .unwrap();

    let extrusion = options.extrusion_options().unwrap();

    assert_approx_eq(
        extrusion.width_for_role(PrintPathRole::ExternalPerimeter),
        0.45,
    );
    assert_approx_eq(extrusion.width_for_role(PrintPathRole::SparseInfill), 0.45);
    assert_approx_eq(extrusion.width_for_role(PrintPathRole::SolidInfill), 0.45);
}

#[test]
fn slice_options_accept_numeric_string_role_filaments() {
    let options: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": [0.4, 0.8],
        "filament_diameter": [1.75, 2.85],
        "wall_filament": "2",
        "sparse_infill_filament": "2",
        "solid_infill_filament": "2",
        "line_width": 0,
        "outer_wall_line_width": 0,
        "sparse_infill_line_width": 0,
        "internal_solid_infill_line_width": 0
    }))
    .unwrap();

    let extrusion = options.extrusion_options().unwrap();

    assert_approx_eq(
        extrusion.width_for_role(PrintPathRole::ExternalPerimeter),
        0.9,
    );
    assert_approx_eq(extrusion.width_for_role(PrintPathRole::SparseInfill), 0.9);
    assert_approx_eq(extrusion.width_for_role(PrintPathRole::SolidInfill), 0.9);
}

#[test]
fn slice_options_accept_float_encoded_integer_role_filaments() {
    let options: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": [0.4, 0.8],
        "filament_diameter": [1.75, 2.85],
        "wall_filament": 2.0,
        "line_width": 0,
        "outer_wall_line_width": 0
    }))
    .unwrap();

    let extrusion = options.extrusion_options().unwrap();

    assert_approx_eq(
        extrusion.width_for_role(PrintPathRole::ExternalPerimeter),
        0.9,
    );
}

#[test]
fn slice_options_reject_invalid_explicit_role_filaments() {
    for (key, value) in [
        ("wall_filament", json!(0)),
        ("wall_filament", json!(-1)),
        ("sparse_infill_filament", json!(1.5)),
        ("solid_infill_filament", json!("2.5")),
        ("solid_infill_filament", json!("fast")),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({ key: value })).unwrap();
        assert!(matches!(
            options.extrusion_options(),
            Err(SliceError::InvalidInput(_))
        ));
    }
}

#[test]
fn slice_options_role_selector_fallback_is_independent_per_hardware_vector() {
    let options: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": [0.4],
        "filament_diameter": [1.75, 2.85],
        "wall_filament": 2,
        "sparse_infill_filament": 2,
        "solid_infill_filament": 3,
        "line_width": 0,
        "outer_wall_line_width": 0,
        "sparse_infill_line_width": 0,
        "internal_solid_infill_line_width": 0
    }))
    .unwrap();

    let extrusion = options.extrusion_options().unwrap();

    assert_approx_eq(
        extrusion.width_for_role(PrintPathRole::ExternalPerimeter),
        0.45,
    );
    assert_approx_eq(extrusion.width_for_role(PrintPathRole::SparseInfill), 0.45);
    assert_approx_eq(extrusion.width_for_role(PrintPathRole::SolidInfill), 0.45);
    assert!(
        extrusion
            .extrusion_delta_for_segment(PrintPathRole::SparseInfill, 0.2, false, 10.0)
            .unwrap()
            < extrusion
                .extrusion_delta_for_segment(PrintPathRole::SolidInfill, 0.2, false, 10.0)
                .unwrap()
    );
}

#[test]
fn slice_options_oversized_role_selector_falls_back_on_all_targets() {
    let options: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": [0.4, 0.8],
        "filament_diameter": [1.75, 2.85],
        "wall_filament": u64::MAX,
        "line_width": 0,
        "outer_wall_line_width": 0
    }))
    .unwrap();

    let extrusion = options.extrusion_options().unwrap();

    assert_approx_eq(
        extrusion.width_for_role(PrintPathRole::ExternalPerimeter),
        0.45,
    );
}
