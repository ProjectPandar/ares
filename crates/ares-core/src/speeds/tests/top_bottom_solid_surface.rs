use super::*;

#[test]
fn top_surface_speed_applies_only_after_first_layer() {
    let options = SpeedOptions::new(120.0, 60.0, 80.0)
        .with_internal_solid_infill_speed(90.0)
        .with_top_surface_speed(45.0)
        .with_first_layer_infill_speed(30.0);

    assert_eq!(
        options.speed_for_role(ToolpathMoveKind::Print, PrintPathRole::TopSolidInfill),
        45.0
    );
    assert_eq!(
        options.speed_for_layer(ToolpathMoveKind::Print, PrintPathRole::TopSolidInfill, true),
        30.0
    );
    assert_eq!(
        options.speed_for_layer(ToolpathMoveKind::Print, PrintPathRole::BottomSurface, true),
        30.0
    );
    assert_eq!(
        options.speed_for_role(ToolpathMoveKind::Print, PrintPathRole::BottomSurface),
        30.0
    );
    assert_eq!(
        options.speed_for_role(ToolpathMoveKind::Print, PrintPathRole::SolidInfill),
        90.0
    );
}

#[test]
fn top_surface_acceleration_and_jerk_do_not_override_first_layer() {
    let acceleration = AccelerationOptions {
        default_mm_s2: 700.0,
        initial_layer_mm_s2: 300.0,
        outer_wall_mm_s2: 450.0,
        bridge_mm_s2: 225.0,
        inner_wall_mm_s2: 650.0,
        travel_mm_s2: 900.0,
        initial_layer_travel_mm_s2: 900.0,
        sparse_infill_mm_s2: 350.0,
        internal_solid_infill_mm_s2: 175.0,
        top_surface_mm_s2: 125.0,
    };
    let jerk = JerkOptions {
        default_mm_s: 8.0,
        initial_layer_mm_s: 6.0,
        outer_wall_mm_s: 7.0,
        inner_wall_mm_s: 4.0,
        infill_mm_s: 5.0,
        top_surface_mm_s: 3.0,
        travel_mm_s: 11.0,
        initial_layer_travel_mm_s: 5.5,
    };

    assert_eq!(
        acceleration.acceleration_for_layer(
            ToolpathMoveKind::Print,
            PrintPathRole::TopSolidInfill,
            false
        ),
        Some(125.0)
    );
    assert_eq!(
        acceleration.acceleration_for_layer(
            ToolpathMoveKind::Print,
            PrintPathRole::TopSolidInfill,
            true
        ),
        Some(300.0)
    );
    assert_eq!(
        acceleration.acceleration_for_layer(
            ToolpathMoveKind::Print,
            PrintPathRole::BottomSurface,
            true
        ),
        Some(300.0)
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
        acceleration.acceleration_for_layer(
            ToolpathMoveKind::Print,
            PrintPathRole::SparseInfill,
            false
        ),
        Some(350.0)
    );
    assert_eq!(
        jerk.jerk_for_layer(
            ToolpathMoveKind::Print,
            PrintPathRole::TopSolidInfill,
            false
        ),
        Some(3.0)
    );
    assert_eq!(
        jerk.jerk_for_layer(ToolpathMoveKind::Print, PrintPathRole::TopSolidInfill, true),
        Some(6.0)
    );
    assert_eq!(
        jerk.jerk_for_layer(ToolpathMoveKind::Print, PrintPathRole::BottomSurface, true),
        Some(6.0)
    );
    assert_eq!(
        jerk.jerk_for_layer(ToolpathMoveKind::Print, PrintPathRole::SolidInfill, false),
        Some(5.0)
    );
    assert_eq!(
        jerk.jerk_for_layer(ToolpathMoveKind::Print, PrintPathRole::SparseInfill, false),
        Some(5.0)
    );
}
