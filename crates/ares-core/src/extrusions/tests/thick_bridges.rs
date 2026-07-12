use super::*;

fn round_6(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

fn base_options() -> ExtrusionOptions {
    ExtrusionOptions::new_for_tests(0.4, 2.0, 0.4, (0.0, 0.0), 0.0)
}

#[test]
fn thick_bridges_default_preserves_existing_bridge_flow() {
    let base = base_options();
    let bridge = base.with_bridge_flow(0.5);
    let perimeter_e = base
        .extrusion_per_mm(PrintPathRole::ExternalPerimeter, 0.2)
        .unwrap();
    let bridge_e = bridge.extrusion_per_mm(PrintPathRole::Bridge, 0.2).unwrap();

    assert_eq!(round_6(bridge_e), round_6(perimeter_e * 0.5));
}

#[test]
fn explicit_false_thick_bridges_preserves_existing_bridge_flow() {
    let base = base_options();
    let bridge = base.with_bridge_flow(0.5).with_thick_bridges(false);
    let perimeter_e = base
        .extrusion_per_mm(PrintPathRole::ExternalPerimeter, 0.2)
        .unwrap();
    let bridge_e = bridge.extrusion_per_mm(PrintPathRole::Bridge, 0.2).unwrap();

    assert_eq!(round_6(bridge_e), round_6(perimeter_e * 0.5));
}

#[test]
fn thick_external_bridge_uses_orca_circular_cross_section() {
    let bridge = base_options().with_thick_bridges(true);
    let bridge_e = bridge.extrusion_per_mm(PrintPathRole::Bridge, 0.2).unwrap();
    let filament_area = std::f64::consts::PI * 1.0_f64.powi(2);
    let expected = 0.4_f64.powi(2) * 0.25 * std::f64::consts::PI / filament_area;

    assert_eq!(round_6(bridge_e), round_6(expected));
}

#[test]
fn thick_bridge_flow_scales_circular_area_by_bridge_flow() {
    let bridge = base_options()
        .with_bridge_flow(0.25)
        .with_thick_bridges(true);
    let bridge_e = bridge.extrusion_per_mm(PrintPathRole::Bridge, 0.2).unwrap();
    let filament_area = std::f64::consts::PI * 1.0_f64.powi(2);
    let expected = 0.25 * 0.4_f64.powi(2) * 0.25 * std::f64::consts::PI / filament_area;

    assert_eq!(round_6(bridge_e), round_6(expected));
}

#[test]
fn thick_bridges_does_not_change_internal_bridge_flow() {
    let base = base_options().with_internal_bridge_flow(0.25);
    let thick = base.with_thick_bridges(true);

    assert_eq!(
        round_6(
            thick
                .extrusion_per_mm(PrintPathRole::InternalBridge, 0.2)
                .unwrap()
        ),
        round_6(
            base.extrusion_per_mm(PrintPathRole::InternalBridge, 0.2)
                .unwrap()
        )
    );
}

#[test]
fn explicit_thick_internal_bridge_uses_orca_circular_cross_section() {
    let bridge = base_options().with_thick_internal_bridges(true);
    let internal_bridge_e = bridge
        .extrusion_per_mm(PrintPathRole::InternalBridge, 0.2)
        .unwrap();
    let filament_area = std::f64::consts::PI * 1.0_f64.powi(2);
    let expected = 0.4_f64.powi(2) * 0.25 * std::f64::consts::PI / filament_area;

    assert_eq!(round_6(internal_bridge_e), round_6(expected));
}

#[test]
fn false_thick_internal_bridges_preserves_current_internal_bridge_flow() {
    let base = base_options().with_internal_bridge_flow(0.25);
    let thin = base.with_thick_internal_bridges(false);
    let perimeter_e = base
        .extrusion_per_mm(PrintPathRole::ExternalPerimeter, 0.2)
        .unwrap();
    let internal_bridge_e = thin
        .extrusion_per_mm(PrintPathRole::InternalBridge, 0.2)
        .unwrap();

    assert_eq!(round_6(internal_bridge_e), round_6(perimeter_e * 0.25));
}

#[test]
fn thick_internal_bridge_flow_scales_circular_area_by_bridge_flow() {
    let bridge = base_options()
        .with_bridge_flow(0.25)
        .with_thick_internal_bridges(true);
    let internal_bridge_e = bridge
        .extrusion_per_mm(PrintPathRole::InternalBridge, 0.2)
        .unwrap();
    let filament_area = std::f64::consts::PI * 1.0_f64.powi(2);
    let expected = 0.25 * 0.4_f64.powi(2) * 0.25 * std::f64::consts::PI / filament_area;

    assert_eq!(round_6(internal_bridge_e), round_6(expected));
}

#[test]
fn thick_internal_bridge_applies_internal_bridge_flow_once() {
    let bridge = base_options()
        .with_bridge_flow(0.5)
        .with_internal_bridge_flow(0.25)
        .with_thick_internal_bridges(true);
    let internal_bridge_e = bridge
        .extrusion_per_mm(PrintPathRole::InternalBridge, 0.2)
        .unwrap();
    let filament_area = std::f64::consts::PI * 1.0_f64.powi(2);
    let expected = 0.5 * 0.25 * 0.4_f64.powi(2) * 0.25 * std::f64::consts::PI / filament_area;

    assert_eq!(round_6(internal_bridge_e), round_6(expected));
}

#[test]
fn thick_internal_bridges_does_not_change_external_bridge_flow() {
    let base = base_options().with_bridge_flow(0.5);
    let thick_internal = base.with_thick_internal_bridges(true);

    assert_eq!(
        round_6(
            thick_internal
                .extrusion_per_mm(PrintPathRole::Bridge, 0.2)
                .unwrap()
        ),
        round_6(base.extrusion_per_mm(PrintPathRole::Bridge, 0.2).unwrap())
    );
}
