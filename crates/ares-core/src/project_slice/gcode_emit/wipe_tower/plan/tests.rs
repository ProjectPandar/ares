use super::*;

#[test]
fn zero_volume_still_reserves_one_wipe_row() {
    assert_eq!(wipe_depth(0.0, (0.5, 0.2), 1.0, 1.5, 38.5), 0.75);
    assert_eq!(wipe_depth(-5.0, (0.5, 0.2), 1.0, 1.5, 38.5), 0.75);
}

#[test]
fn first_layer_depth_uses_flow_spacing_instead_of_later_extra_spacing() {
    let mut settings = crate::ProjectSettings::default();
    settings.process.print.wipe_tower_extra_spacing.0 = 200.0;
    settings.printer.gcode.enable_filament_ramming.0 = false;
    let filaments = (0..2)
        .map(|index| Filament::from_config(&settings, index))
        .collect::<Vec<_>>();
    let mut plan = Plan::new(&settings, &filaments);
    plan.add((0.2, 0.2), (0, 1), 100.0, &filaments);
    plan.add((0.4, 0.2), (1, 0), 100.0, &filaments);
    assert_eq!(plan.layers[0].changes[0].ramming_depth, 0.0);
    assert_eq!(
        plan.layers[1].changes[0].required_depth,
        2.0 * plan.layers[0].changes[0].required_depth
    );
}

#[test]
fn deeper_upper_layer_propagates_through_shallower_layers() {
    let mut settings = crate::ProjectSettings::default();
    settings.process.print.prime_tower_width.0 = 40.0;
    settings.printer.gcode.enable_filament_ramming.0 = false;
    let filaments = vec![Filament::from_config(&settings, 0)];
    let mut plan = Plan::new(&settings, &filaments);
    plan.add((0.2, 0.2), (0, 0), 0.0, &filaments);
    plan.add((0.4, 0.2), (0, 1), 100.0, &filaments);
    plan.plan_tower();
    assert_eq!(plan.first_layer, Some(0));
    assert_eq!(plan.layers[0].depth, plan.layers[1].changes_depth());
    assert_eq!(plan.depth, plan.layers[1].depth + plan.perimeter_width);
    assert_eq!(plan.height, 0.4);
}
