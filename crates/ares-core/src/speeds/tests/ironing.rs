use super::*;

#[test]
fn ironing_speed_defaults_to_orca_20_mm_s_without_changing_top_surface() {
    let options = SpeedOptions::new(120.0, 60.0, 80.0).with_top_surface_speed(45.0);

    assert_eq!(
        options.speed_for_role(ToolpathMoveKind::Print, PrintPathRole::Ironing),
        20.0
    );
    assert_eq!(
        options.speed_for_role(ToolpathMoveKind::Print, PrintPathRole::TopSolidInfill),
        45.0
    );
}

#[test]
fn configured_ironing_speed_is_independent_from_top_surface_speed() {
    let options = SpeedOptions::new(120.0, 60.0, 80.0)
        .with_top_surface_speed(70.0)
        .with_ironing_speed(15.0);

    assert_eq!(
        options.speed_for_role(ToolpathMoveKind::Print, PrintPathRole::Ironing),
        15.0
    );
    assert_eq!(
        options.speed_for_role(ToolpathMoveKind::Print, PrintPathRole::TopSolidInfill),
        70.0
    );
}

#[test]
fn first_layer_ironing_keeps_initial_layer_infill_speed() {
    let options = SpeedOptions::new(120.0, 60.0, 80.0)
        .with_ironing_speed(15.0)
        .with_first_layer_infill_speed(33.0);

    assert_eq!(
        options.speed_for_layer(ToolpathMoveKind::Print, PrintPathRole::Ironing, true),
        33.0
    );
    assert_eq!(
        options.speed_for_layer(ToolpathMoveKind::Print, PrintPathRole::Ironing, false),
        15.0
    );
}
