use super::*;

/// KSR oracle calibration (`case-u7sdch`): line_width 0.45 → support
/// flow spacing ≈ 0.45 − 0.2·(1 − π/4) ≈ 0.407; base pattern
/// spacing 0.2; interface spacing 0.2; support_angle 0; raft 100
/// layers (base 50 / interface 50).
fn ksr_inputs() -> RaftFillInputs {
    RaftFillInputs {
        support_flow_spacing: 0.407_086_4,
        interface_flow_spacing: 0.407_086_4,
        base_pattern_spacing: 0.2,
        interface_spacing: 0.2,
        support_angle_degrees: 0.0,
        base_raft_layers: 50,
        interface_raft_layers: 50,
        with_sheath: false,
        honeycomb_base: false,
    }
}

/// `support_spacing = 0.2 + 0.407`, `support_density = 0.407/0.607`
/// (`SupportParameters.hpp:120-125`).
#[test]
fn ksr_densities() {
    let params = ksr_inputs().derive();

    assert!((params.support_density - 0.407_086_4 / 0.607_086_4).abs() < 1e-9);
    assert!((params.raft_interface_density - 0.407_086_4 / 0.607_086_4).abs() < 1e-9);
    // Both densities < 0.95, no sheath, non-honeycomb → SupportBase.
    assert_eq!(params.base_fill_pattern, RaftFillPattern::SupportBase);
    assert_eq!(params.interface_fill_pattern, RaftFillPattern::SupportBase);
}

/// KSR raft angles: base_raft_layers > 1 → flange = 90°, base = 0°,
/// interface = 90° + 90° (interface_raft_layers even) = 180° ≡ 0°
/// direction; the oracle layer 1 flange extrudes vertically (X
/// constant) and the base layers horizontally (Y constant)
/// (`SupportParameters.hpp:153-158`, oracle lines 118-135 / 294+).
#[test]
fn ksr_angles() {
    let params = ksr_inputs().derive();

    assert!((params.flange_angle - FRAC_PI_2).abs() < 1e-9);
    assert!(params.base_angle.abs() < 1e-9);
    // interface_raft_layers = 50 (even) → +90°.
    assert!((params.interface_angle - PI).abs() < 1e-9);
    // Per-layer alternation ±45° (`:284-285`).
    assert!((params.interface_angle_for_layer(0) - (PI + FRAC_PI_4)).abs() < 1e-9);
    assert!((params.interface_angle_for_layer(1) - (PI - FRAC_PI_4)).abs() < 1e-9);
    // Directions normalize: 5π/4 ≡ π/4, 3π/4 stays.
    assert!((normalized_direction(params.interface_angle_for_layer(0)) - FRAC_PI_4).abs() < 1e-9);
    assert!(
        (normalized_direction(params.interface_angle_for_layer(1)) - 3.0 * FRAC_PI_4).abs() < 1e-9
    );
}

/// `raft_layers == 2|3` branch: flange = base_angle, interface =
/// interface_angle + 90° (`:159-165`).
#[test]
fn two_layer_raft_angles() {
    let params = RaftFillInputs {
        base_raft_layers: 1,
        interface_raft_layers: 1,
        ..ksr_inputs()
    }
    .derive();

    assert!(params.flange_angle.abs() < 1e-9);
    assert!((params.interface_angle - PI).abs() < 1e-9);
}

/// `raft_layers == 1`: the single contact layer prints at 90°
/// (`:166-171`).
#[test]
fn single_layer_raft_angle() {
    let params = RaftFillInputs {
        base_raft_layers: 0,
        interface_raft_layers: 1,
        ..ksr_inputs()
    }
    .derive();

    assert!((params.flange_angle - FRAC_PI_2).abs() < 1e-9);
    assert!((params.interface_angle - FRAC_PI_2).abs() < 1e-9);
}

/// Dense supports (spacing 0) promote both patterns to Rectilinear
/// (`:133-135`).
#[test]
fn solid_density_promotes_rectilinear() {
    let params = RaftFillInputs {
        base_pattern_spacing: 0.0,
        interface_spacing: 0.0,
        ..ksr_inputs()
    }
    .derive();

    assert_eq!(params.base_fill_pattern, RaftFillPattern::Rectilinear);
    assert_eq!(params.interface_fill_pattern, RaftFillPattern::Rectilinear);
    assert!((params.support_density - 1.0).abs() < 1e-9);
    assert!((params.raft_interface_density - 1.0).abs() < 1e-9);
}

/// Honeycomb base overrides the density rule (`:132-134`).
#[test]
fn honeycomb_base_wins() {
    let params = RaftFillInputs {
        honeycomb_base: true,
        ..ksr_inputs()
    }
    .derive();

    assert_eq!(params.base_fill_pattern, RaftFillPattern::Honeycomb);
    assert_eq!(params.interface_fill_pattern, RaftFillPattern::SupportBase);
}
