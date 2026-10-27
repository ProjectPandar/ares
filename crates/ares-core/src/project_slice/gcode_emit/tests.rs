use super::motion::MotionOptions;
use crate::{Nullable, OrcaFloat};

mod extruder_offset;
mod filament_end;
mod filament_start;
mod filament_stats;
mod flow_ratio;
mod layer_placeholders;
mod object_labels;
mod runtime_placeholders;
mod scarf;
mod spiral_vase;
mod timelapse;
mod wrapping;

#[test]
fn ksr_motion_options_resolve_from_typed_project_settings() {
    let prepared = crate::project_slice::perimeters::prepare_post_classic_traversal(
        crate::project_slice::tests::support::ksr_project(),
    )
    .unwrap();

    let options = MotionOptions::from_traversal(&prepared, 0);

    assert_eq!(options.travel_feedrate, 60_000.0);
    assert_eq!(options.first_layer_travel_feedrate, 60_000.0);
    assert_eq!(
        options.filament_area,
        1.75_f64 * 1.75_f64 * 0.25 * std::f64::consts::PI
    );
    assert_eq!(options.filament_flow_ratio, 0.98);
    assert_eq!(options.print_flow_ratio, 1.0);
    assert_eq!(options.max_volumetric_speed, 21.0);
    assert_eq!(options.initial_layer_speed, 50.0);
    assert_eq!(options.initial_layer_infill_speed, 105.0);
    assert_eq!(options.inner_wall_speed, 300.0);
    assert_eq!(options.outer_wall_speed, 200.0);
    assert_eq!(options.bridge_speed, 50.0);
    assert_eq!(options.internal_bridge_speed, 75.0);
    assert_eq!(options.sparse_infill_speed, 270.0);
    assert_eq!(options.internal_solid_infill_speed, 250.0);
    assert_eq!(options.top_surface_speed, 200.0);
    assert_eq!(options.gap_infill_speed, 250.0);
    assert_eq!(options.initial_layer_acceleration, 500.0);
    assert_eq!(options.default_acceleration, 10_000.0);
    assert_eq!(options.outer_wall_acceleration, 5_000.0);
    assert_eq!(options.bridge_acceleration, 2_500.0);
    assert_eq!(options.top_surface_acceleration, 2_000.0);
    assert_eq!(options.initial_layer_travel_acceleration, 6_000.0);
    assert_eq!(options.travel_acceleration, 10_000.0);
    assert_eq!(options.retraction_length, 0.4);
    assert_eq!(options.deretraction_feedrate, 1_800.0);
    assert_eq!(options.z_hop, 0.4);
    assert_eq!(options.retraction_feedrate, 1_800.0);
    assert!(options.wipe);
    assert_eq!(options.wipe_distance, 1.0);
    assert_eq!(options.retraction_minimum_travel, 1.0);
    assert!(options.reduce_infill_retraction);
    assert_eq!(options.retract_before_wipe, 0.0);
    assert!(options.role_based_wipe_speed);
    assert_eq!(options.z_hop_type, crate::ZHopType::Spiral);
    assert_eq!(options.travel_slope_radians, 0.05235987755982988);
    assert_eq!(options.seam_gap, 0.04);
}

#[test]
fn nullable_filament_flow_ratio_uses_selected_value() {
    assert_eq!(
        super::motion::first_nullable_float(
            &[Nullable::Nil, Nullable::Value(OrcaFloat(0.97)),],
            1.0
        ),
        0.97,
    );
}

#[tokio::test]
async fn disable_m73_true_suppresses_project_progress_lines() {
    let mut archive = crate::project_slice::tests::support::KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"disable_m73\": \"0\"",
        "\"disable_m73\": \"1\"",
    );

    let output = crate::slice_project(
        &archive.bytes(),
        crate::project_slice::tests::support::metadata(),
        crate::SlicerContext::new(0),
    )
    .await
    .unwrap();
    let output = std::str::from_utf8(&output).unwrap();

    assert!(!output.lines().any(|line| line.starts_with("M73 P")));
}
