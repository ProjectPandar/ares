use super::motion::MotionOptions;
use crate::{Nullable, OrcaFloat};

#[test]
fn ksr_motion_options_resolve_from_typed_project_settings() {
    let prepared = crate::project_slice::perimeters::prepare_post_classic_traversal(
        crate::project_slice::tests::support::ksr_project(),
    )
    .unwrap();

    let options = MotionOptions::from_traversal(&prepared);

    assert_eq!(options.travel_feedrate, 60_000.0);
    assert_eq!(options.first_layer_travel_feedrate, 60_000.0);
    assert_eq!(
        options.filament_area,
        std::f64::consts::PI * 1.75_f64.powi(2) * 0.25
    );
    assert_eq!(options.filament_flow_ratio, 0.98);
    assert_eq!(options.max_volumetric_speed, 21.0);
    assert_eq!(options.initial_layer_speed, 50.0);
    assert_eq!(options.initial_layer_infill_speed, 105.0);
    assert_eq!(options.inner_wall_speed, 300.0);
    assert_eq!(options.outer_wall_speed, 200.0);
    assert_eq!(options.sparse_infill_speed, 270.0);
    assert_eq!(options.internal_solid_infill_speed, 250.0);
    assert_eq!(options.top_surface_speed, 200.0);
    assert_eq!(options.gap_infill_speed, 250.0);
    assert_eq!(options.initial_layer_acceleration, 500);
    assert_eq!(options.default_acceleration, 10_000);
    assert_eq!(options.outer_wall_acceleration, 5_000);
    assert_eq!(options.top_surface_acceleration, 2_000);
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
async fn ksr_project_motion_is_finite_and_uses_configured_first_layer_rates() {
    let output = crate::slice_project(
        crate::project_slice::tests::support::ksr_project(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let output = std::str::from_utf8(&output).unwrap();

    assert!(!output.contains("Einf"));
    assert!(!output.contains("Enan"));
    assert!(!output.contains(" F0\n"));
    assert!(output.contains("G1 X144.504 Y100.092 F60000\n"));
    assert!(output.contains("; FEATURE: Inner wall\n; LINE_WIDTH: 0.5\nG1 F3000\n"));
}

#[tokio::test]
async fn ksr_project_emits_3mf_object_labels_per_layer() {
    let output = crate::slice_project(
        crate::project_slice::tests::support::ksr_project(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let output = std::str::from_utf8(&output).unwrap();

    assert_eq!(
        output
            .matches("; printing object ksr_fdmtest_v4.drc id:2 copy 0\n")
            .count(),
        460
    );
    assert_eq!(
        output
            .matches("; start printing object, unique label id: 133\nM624 AQAAAAAAAAA=\n")
            .count(),
        460
    );
    assert_eq!(
        output
            .matches("; stop printing object, unique label id: 133\nM625\n")
            .count(),
        460
    );
}
