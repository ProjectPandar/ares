use std::f64::consts::PI;

use crate::project_slice::prepare_infill::bridge_over_infill::automatic_bridge_angle::{
    direction_window_for_test, reduce_directions_for_test,
};

#[test]
fn task22o51_closed_window_endpoints_and_strict_score_ties_match_oracle() {
    let delta = PI * 0.1;
    let (ordinary, ordered) =
        reduce_directions_for_test(&[(PI - delta, 1), (PI, 1), (PI + delta, 1)]);
    assert_eq!(ordinary.to_bits(), 0x4009_21fb_5444_2d18);
    assert_eq!(ordered.len(), 3);

    let (tie, _) = reduce_directions_for_test(&[(0.75 * PI, 2), (1.25 * PI, 2)]);
    assert_eq!(tie.to_bits(), 0x4002_d97c_7f33_21d2);
}

#[test]
fn task22o51_lower_and_synthetic_upper_periodic_wraps_match_oracle() {
    let (lower, _) = reduce_directions_for_test(&[(0.55 * PI, 1), (1.45 * PI, 1)]);
    assert_eq!(lower.to_bits(), 0x3ff9_21fb_5444_2d18);

    let synthetic = [(0.15 * PI, 1), (1.75 * PI, 1)];
    let (upper, _) = reduce_directions_for_test(&synthetic);
    assert_eq!(upper.to_bits(), 0x3ff6_9e95_6570_8efc);
    let (weighted, score) = direction_window_for_test(&synthetic, 1.75 * PI);
    assert_eq!(weighted.to_bits(), 0x4022_38a3_037e_3a4b);
    assert_eq!(score, 2);
    assert_eq!(
        (weighted / f64::from(score)).to_bits(),
        0x4012_38a3_037e_3a4b
    );
}

#[test]
fn task22o51_numeric_key_equivalence_coalesces_signed_zero() {
    let (angle, ordered) = reduce_directions_for_test(&[(-0.0, 1), (0.0, 2)]);
    assert_eq!(angle.to_bits(), 0);
    assert_eq!(ordered, vec![(0x8000_0000_0000_0000, 3)]);
}
