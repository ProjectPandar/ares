use super::*;
use crate::{ExtrusionMove, Point2};

#[test]
fn solid_infill_uses_internal_solid_speed_and_first_layer_infill_speed() {
    let options = SpeedOptions::new(120.0, 60.0, 80.0)
        .with_internal_solid_infill_speed(100.0)
        .with_first_layer_infill_speed(45.0);

    assert_eq!(
        options.speed_for_role(ToolpathMoveKind::Print, PrintPathRole::SparseInfill),
        80.0
    );
    assert_eq!(
        options.speed_for_role(ToolpathMoveKind::Print, PrintPathRole::SolidInfill),
        100.0
    );
    assert_eq!(
        options.speed_for_layer(ToolpathMoveKind::Print, PrintPathRole::SolidInfill, true),
        45.0
    );
}

#[test]
fn solid_infill_uses_internal_solid_acceleration_and_infill_jerk() {
    let acceleration = AccelerationOptions {
        default_mm_s2: 700.0,
        initial_layer_mm_s2: 0.0,
        outer_wall_mm_s2: 450.0,
        bridge_mm_s2: 225.0,
        inner_wall_mm_s2: 650.0,
        travel_mm_s2: 900.0,
        initial_layer_travel_mm_s2: 900.0,
        sparse_infill_mm_s2: 350.0,
        internal_solid_infill_mm_s2: 175.0,
        top_surface_mm_s2: 700.0,
    };
    let jerk = JerkOptions {
        default_mm_s: 8.0,
        initial_layer_mm_s: 0.0,
        outer_wall_mm_s: 7.0,
        inner_wall_mm_s: 4.0,
        infill_mm_s: 5.0,
        top_surface_mm_s: 5.0,
        travel_mm_s: 11.0,
        initial_layer_travel_mm_s: 5.5,
    };

    assert_eq!(
        acceleration.acceleration_for_layer(
            ToolpathMoveKind::Print,
            PrintPathRole::SparseInfill,
            false
        ),
        Some(350.0)
    );
    assert_eq!(
        acceleration.acceleration_for_layer(
            ToolpathMoveKind::Print,
            PrintPathRole::SolidInfill,
            false
        ),
        Some(175.0)
    );
    assert_eq!(
        jerk.jerk_for_layer(ToolpathMoveKind::Print, PrintPathRole::SolidInfill, false),
        Some(5.0)
    );
}

#[test]
fn solid_infill_reuses_sparse_slowdown_reference() {
    let layers = [LayerExtrusionMoves::new(
        1,
        0.4,
        vec![ExtrusionMove::new(
            ToolpathMoveKind::Print,
            PrintPathRole::SolidInfill,
            Point2::new(2.0, 0.0),
            Some(0.2),
        )],
        0.2,
    )];
    let options = SpeedOptions::new(120.0, 150.0, 150.0)
        .with_first_layer_infill_speed(60.0)
        .with_slow_down_layers(4);

    let output = generate_speed_moves(&layers, options);

    assert_eq!(output[0].moves()[0].speed_mm_s(), 82.5);
}

#[test]
fn solid_infill_reuses_existing_volumetric_speed_cap() {
    let layer = LayerExtrusionMoves::new(
        0,
        0.2,
        vec![
            ExtrusionMove::new(
                ToolpathMoveKind::Travel,
                PrintPathRole::SolidInfill,
                Point2::new(0.0, 0.0),
                None,
            ),
            ExtrusionMove::new(
                ToolpathMoveKind::Print,
                PrintPathRole::SolidInfill,
                Point2::new(10.0, 0.0),
                Some(1.0),
            ),
        ],
        1.0,
    );
    let options = SpeedOptions::new(120.0, 100.0, 100.0)
        .with_first_layer_infill_speed(100.0)
        .with_filament_diameter(2.0)
        .with_filament_max_volumetric_speed(1.0);

    let output = generate_speed_moves(&[layer], options);

    assert!((output[0].moves()[1].speed_mm_s() - 3.183098861837907).abs() <= 0.000000001);
    assert!((output[0].moves()[1].feedrate_mm_min() - 190.9859317102744).abs() <= 0.0000001);
}
