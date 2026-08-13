use crate::{OrcaBool, OrcaFloat, ProjectSettings, RegionOptions};

use super::apply_internal_bridge_angle_override;

#[test]
fn task22o49_nonpositive_and_nan_override_return_detected_bits_without_arithmetic() {
    for override_angle in [0.0, -0.0, -17.3, f64::NEG_INFINITY, f64::NAN] {
        let mut region = region();
        region.internal_bridge_angle = OrcaFloat(override_angle);
        region.relative_bridge_angle = OrcaBool(true);
        region.align_infill_direction_to_model = OrcaBool(true);
        let before = region_bits(&region);
        let before_debug = format!("{region:?}");
        let detected = f64::from_bits(0x7ff8_0000_0000_0042);

        let first = apply_internal_bridge_angle_override(detected, &region, f64::NAN);
        let second = apply_internal_bridge_angle_override(detected, &region, f64::NAN);

        assert_eq!(first.to_bits(), detected.to_bits());
        assert_eq!(second.to_bits(), detected.to_bits());
        assert_eq!(region_bits(&region), before);
        assert_eq!(format!("{region:?}"), before_debug);
    }
}

#[test]
fn task22o49_positive_override_preserves_source_degree_to_radian_order() {
    let mut region = region();
    region.internal_bridge_angle = OrcaFloat(17.3);
    let before = region_bits(&region);
    let before_debug = format!("{region:?}");

    let angle = apply_internal_bridge_angle_override(4.0, &region, f64::NAN);

    assert_eq!(std::f64::consts::PI.to_bits(), 0x4009_21fb_5444_2d18);
    assert_eq!(angle.to_bits(), 0x3fd3_5304_5f82_ed32);
    assert_ne!(angle.to_bits(), 17.3_f64.to_radians().to_bits());
    assert_eq!(region_bits(&region), before);
    assert_eq!(format!("{region:?}"), before_debug);
}

#[test]
fn task22o49_relative_adds_detected_angle_and_ignores_alignment_rotation() {
    let mut region = region();
    region.internal_bridge_angle = OrcaFloat(17.3);
    region.relative_bridge_angle = OrcaBool(true);
    region.align_infill_direction_to_model = OrcaBool(true);
    let before = region_bits(&region);
    let before_debug = format!("{region:?}");

    let first = apply_internal_bridge_angle_override(0.25, &region, f64::NAN);
    let second = apply_internal_bridge_angle_override(0.25, &region, f64::NAN);

    assert_eq!(first.to_bits(), 0x3fe1_a982_2fc1_7699);
    assert_eq!(first.to_bits(), second.to_bits());
    assert_eq!(region_bits(&region), before);
    assert_eq!(format!("{region:?}"), before_debug);
}

#[test]
fn task22o49_absolute_alignment_replaces_detected_and_does_not_normalize() {
    let mut region = region();
    region.internal_bridge_angle = OrcaFloat(17.3);
    region.align_infill_direction_to_model = OrcaBool(true);
    let before = region_bits(&region);
    let before_debug = format!("{region:?}");

    let aligned = apply_internal_bridge_angle_override(
        f64::from_bits(0x7ff8_0000_0000_0042),
        &region,
        std::f64::consts::FRAC_PI_2,
    );
    assert_eq!(aligned.to_bits(), 0x3ffd_f6bc_6c24_e864);
    assert_eq!(region_bits(&region), before);
    assert_eq!(format!("{region:?}"), before_debug);

    region.relative_bridge_angle = OrcaBool(true);
    let relative_before = format!("{region:?}");
    let negative = apply_internal_bridge_angle_override(-10.0, &region, 0.0);
    let above_turn = apply_internal_bridge_angle_override(7.0, &region, 0.0);
    assert!(negative < 0.0);
    assert!(above_turn > 2.0 * std::f64::consts::PI);
    assert_eq!(region_bits(&region)[0], before[0]);
    assert_eq!(format!("{region:?}"), relative_before);
}

#[test]
fn task22o49_nonfinite_arithmetic_obeys_branch_ownership() {
    let mut region = region();
    region.internal_bridge_angle = OrcaFloat(17.3);
    let absolute_before = format!("{region:?}");

    let absolute_replaces_nan = apply_internal_bridge_angle_override(f64::NAN, &region, f64::NAN);
    assert_eq!(absolute_replaces_nan.to_bits(), 0x3fd3_5304_5f82_ed32);
    assert_eq!(format!("{region:?}"), absolute_before);

    region.relative_bridge_angle = OrcaBool(true);
    let relative_before = format!("{region:?}");
    let first_nan = apply_internal_bridge_angle_override(f64::NAN, &region, f64::NAN);
    let second_nan = apply_internal_bridge_angle_override(f64::NAN, &region, f64::NAN);
    assert!(first_nan.is_nan());
    assert!(second_nan.is_nan());
    assert_eq!(
        apply_internal_bridge_angle_override(0.0, &region, f64::NAN).to_bits(),
        0x3fd3_5304_5f82_ed32
    );
    assert_eq!(format!("{region:?}"), relative_before);

    region.relative_bridge_angle = OrcaBool(false);
    region.align_infill_direction_to_model = OrcaBool(true);
    let aligned_before = format!("{region:?}");
    assert!(apply_internal_bridge_angle_override(0.0, &region, f64::NAN).is_nan());
    assert_eq!(format!("{region:?}"), aligned_before);

    region.internal_bridge_angle = OrcaFloat(f64::INFINITY);
    region.align_infill_direction_to_model = OrcaBool(false);
    let infinite_before = format!("{region:?}");
    assert_eq!(
        apply_internal_bridge_angle_override(0.0, &region, 0.0),
        f64::INFINITY
    );
    assert_eq!(format!("{region:?}"), infinite_before);
    region.relative_bridge_angle = OrcaBool(true);
    let relative_infinite_before = format!("{region:?}");
    assert!(apply_internal_bridge_angle_override(f64::NEG_INFINITY, &region, 0.0).is_nan());
    assert_eq!(format!("{region:?}"), relative_infinite_before);
}

fn region() -> RegionOptions {
    RegionOptions::from_base(&ProjectSettings::default().process.region)
}

fn region_bits(region: &RegionOptions) -> [u64; 3] {
    [
        region.internal_bridge_angle.0.to_bits(),
        u64::from(region.relative_bridge_angle.0),
        u64::from(region.align_infill_direction_to_model.0),
    ]
}
