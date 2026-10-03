use super::{CoordinateScale, RadiusQuantization};

#[test]
fn radius_buckets_preserve_linear_transition_and_exponential_truncation() {
    let quantization = RadiusQuantization::new(CoordinateScale::Normal, 200_001, 0);
    for (radius, expected) in [
        (0, 0),
        (1, 200_001),
        (200_001, 200_001),
        (200_002, 1_500_000),
        (1_500_000, 1_500_000),
        (1_500_001, 2_799_999),
        (2_799_999, 2_799_999),
        (2_800_000, 4_199_998),
    ] {
        assert_eq!(quantization.ceil(radius, true), expected, "radius={radius}");
    }
}

#[test]
fn radius_buckets_use_collision_resolution_at_threshold_boundary() {
    let quantization = RadiusQuantization::new(CoordinateScale::Normal, 1_000_000, 0);
    assert_eq!(quantization.ceil(1_000_001, true), 1_500_000);
    assert_eq!(quantization.ceil(1_500_001, true), 2_250_000);
    let quantization = RadiusQuantization::new(CoordinateScale::Normal, 3_000_000, 0);
    assert_eq!(quantization.ceil(3_000_001, true), 3_500_000);
    assert_eq!(quantization.ceil(3_500_001, true), 5_250_000);
}

#[test]
fn regular_xy_zero_radius_is_exact_and_next_ceil_removes_clearance_delta() {
    let quantization = RadiusQuantization::new(CoordinateScale::Normal, 200_000, 250_000);
    assert_eq!(quantization.ceil(0, true), 0);
    assert_eq!(quantization.ceil(0, false), 250_000);
    assert_eq!(quantization.ceil(1, false), 1_500_000);
    assert_eq!(quantization.next_ceil(1, false), 1_250_000);
    assert_eq!(quantization.next_ceil(1, true), 200_000);
}

#[test]
fn ignored_buckets_are_skipped_except_for_base_and_exact_zero() {
    let mut quantization = RadiusQuantization::new(CoordinateScale::Normal, 200_000, 0);
    quantization.ignored_radii = vec![200_000, 1_500_000, 2_800_000];
    assert_eq!(quantization.ceil(0, true), 0);
    assert_eq!(quantization.ceil(1, true), 200_000);
    assert_eq!(quantization.ceil(200_001, true), 4_200_000);
}

#[test]
fn large_bed_radius_buckets_use_the_same_coordinate_lattice_as_geometry() {
    let quantization = RadiusQuantization::new(CoordinateScale::LargeBed, 20_000, 25_000);
    assert_eq!(quantization.ceil(0, false), 25_000);
    assert_eq!(quantization.ceil(20_001, true), 150_000);
    assert_eq!(quantization.ceil(150_001, true), 280_000);
    assert_eq!(quantization.ceil(280_001, true), 420_000);
}
