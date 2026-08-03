use crate::{SliceError, geometry::CoordinateScale};

use super::super::{
    preflight::{
        TestSurfaceInput, convert_overlap_for_test, extra_perimeters_active_for_test,
        validate_surface_for_test,
    },
    types::NoOverlapOffset,
};

const RANGE_ERROR: &str =
    "Classic infill-boundary overlap is outside the supported coordinate range";

#[test]
fn task22o15_overlap_preserves_signed_percent_and_scale_modes() {
    for (scale, basis, percent, expected) in [
        (CoordinateScale::Normal, 1_000_003, 15.25, 152_500),
        (CoordinateScale::LargeBed, 1_000_003, 15.25, 152_500),
        (CoordinateScale::Normal, 1_000_003, -12.5, -125_000),
    ] {
        assert_eq!(
            convert_overlap_for_test(basis, percent, scale).unwrap(),
            expected
        );
    }
}

#[test]
fn task22o15_overlap_rejects_each_non_representable_conversion() {
    for (scale, percent) in [
        (CoordinateScale::Normal, f64::MAX),
        (CoordinateScale::LargeBed, f64::MAX),
        (CoordinateScale::Normal, -f64::MAX),
    ] {
        assert_range_error(convert_overlap_for_test(i64::MAX / 2, percent, scale));
    }
}

#[test]
fn task22o15_inset_and_layer_option_selection_are_literal() {
    let no_loops = validate_surface_for_test(input(-1, 7, true, 15.0, 25.0)).unwrap();
    assert_eq!(overlap_tuple(no_loops), (0, 0, 0, 600));

    let one_loop_first = validate_surface_for_test(input(0, 0, true, 15.0, 25.0)).unwrap();
    assert_eq!(overlap_tuple(one_loop_first), (63, 187, 0, 600));

    let many_middle = validate_surface_for_test(input(1, 7, true, 15.0, 25.0)).unwrap();
    assert_eq!(overlap_tuple(many_middle), (74, 101, 168, 600));

    let many_last = validate_surface_for_test(input(1, 7, false, 15.0, 25.0)).unwrap();
    assert_eq!(overlap_tuple(many_last), (7, 168, 0, 600));
}

#[test]
fn task22o15_odd_halves_and_both_no_overlap_branches_are_literal() {
    let two = validate_surface_for_test(TestSurfaceInput {
        external_spacing: 501,
        perimeter_spacing: 351,
        solid_infill_spacing: 1_003,
        ..input(1, 4, true, 10.0, 20.0)
    })
    .unwrap();
    assert_eq!(two.overlap.inset, 108);
    assert_eq!(two.overlap.min_perimeter_infill_spacing, 601);
    assert_eq!(two.ordinary_first.to_bits(), (-408.5_f32).to_bits());
    assert_eq!(two.ordinary_second.to_bits(), 300.5_f32.to_bits());
    assert!(matches!(two.no_overlap, NoOverlapOffset::Two { second, .. } if second == 233.0));

    let one = validate_surface_for_test(TestSurfaceInput {
        external_spacing: 501,
        perimeter_spacing: 351,
        solid_infill_spacing: 1_003,
        ..input(1, 4, true, 100.0, 20.0)
    })
    .unwrap();
    assert!(matches!(one.no_overlap, NoOverlapOffset::One { delta } if delta == -175.0));
}

#[test]
fn task22o15_post_subtraction_and_no_overlap_delta_overflows_are_stable() {
    let base = TestSurfaceInput {
        loop_number: 1,
        external_spacing: 1 << 60,
        perimeter_spacing: 1 << 60,
        solid_infill_spacing: i64::MAX,
        layer_id: 7,
        has_upper: true,
        ordinary_percent: 0.0,
        top_percent: 0.0,
        scale: CoordinateScale::Normal,
    };
    let pre_inset = base.perimeter_spacing / 2;
    let basis = pre_inset + base.solid_infill_spacing / 2;

    let mut post_overflow = base;
    post_overflow.ordinary_percent = i64::MIN as f64 / basis as f64 * 100.0;
    assert_range_error(validate_surface_for_test(post_overflow));

    let mut no_overlap_overflow = base;
    let target_overlap = -7.25 * (1_u64 << 60) as f64;
    no_overlap_overflow.ordinary_percent = target_overlap / basis as f64 * 100.0;
    assert_range_error(validate_surface_for_test(no_overlap_overflow));
}

#[test]
fn task22o15_extra_perimeter_guard_requires_every_source_operand() {
    let active = [0, 1, 1, 1, 2, 3, 0];
    assert!(extra_perimeters_active_for_test(active));
    for index in [1, 2, 3] {
        let mut inactive = active;
        inactive[index] = 0;
        assert!(!extra_perimeters_active_for_test(inactive));
    }
    for inactive in [
        [1, 1, 1, 1, 2, 3, 0],
        [0, 1, 1, 1, 0, 3, 0],
        [0, 1, 1, 1, 2, 0, 0],
    ] {
        assert!(!extra_perimeters_active_for_test(inactive));
    }
}

#[test]
fn task22o15_source_derived_extrema_keep_negation_and_else_delta_representable() {
    let minimal = validate_surface_for_test(TestSurfaceInput {
        loop_number: -1,
        external_spacing: i64::MAX,
        perimeter_spacing: i64::MAX,
        solid_infill_spacing: i64::MAX,
        layer_id: 0,
        has_upper: false,
        ordinary_percent: f64::MAX,
        top_percent: f64::MAX,
        scale: CoordinateScale::Normal,
    })
    .unwrap();
    assert_eq!(minimal.overlap.inset, 0);
    assert_eq!(minimal.overlap.infill_peri_overlap, 0);

    let pre_inset = i64::MAX / 2;
    let basis = pre_inset + i64::MAX / 2;
    let target_overlap = (i64::MAX - 4_095) as f64;
    let boundary = validate_surface_for_test(TestSurfaceInput {
        loop_number: 1,
        external_spacing: i64::MAX,
        perimeter_spacing: i64::MAX,
        solid_infill_spacing: i64::MAX,
        layer_id: 1,
        has_upper: true,
        ordinary_percent: target_overlap / basis as f64 * 100.0,
        top_percent: 0.0,
        scale: CoordinateScale::Normal,
    })
    .unwrap();
    assert_eq!(boundary.overlap.infill_peri_overlap, target_overlap as i64);
    assert!(boundary.overlap.inset < 0);
    assert!(boundary.overlap.inset.checked_neg().is_some());
    assert!(
        boundary.overlap.infill_peri_overlap > boundary.overlap.min_perimeter_infill_spacing / 2
    );
    let expected_else = boundary
        .overlap
        .inset
        .checked_neg()
        .and_then(|value| value.checked_sub(boundary.overlap.infill_peri_overlap))
        .unwrap();
    assert_eq!(expected_else, -pre_inset);
    match boundary.no_overlap {
        NoOverlapOffset::One { delta } => assert_eq!(delta, expected_else as f64 as f32),
        NoOverlapOffset::Two { .. } => panic!("maximal overlap did not select the else branch"),
    }
}

fn input(
    loop_number: i32,
    layer_id: usize,
    has_upper: bool,
    ordinary_percent: f64,
    top_percent: f64,
) -> TestSurfaceInput {
    TestSurfaceInput {
        loop_number,
        external_spacing: 500,
        perimeter_spacing: 350,
        solid_infill_spacing: 1_000,
        layer_id,
        has_upper,
        ordinary_percent,
        top_percent,
        scale: CoordinateScale::Normal,
    }
}

fn overlap_tuple(surface: super::super::types::ValidatedSurface) -> (i64, i64, i64, i64) {
    (
        surface.overlap.inset,
        surface.overlap.infill_peri_overlap,
        surface.overlap.top_infill_peri_overlap,
        surface.overlap.min_perimeter_infill_spacing,
    )
}

fn assert_range_error<T>(result: Result<T, SliceError>) {
    assert!(matches!(result, Err(SliceError::InvalidInput(message)) if message == RANGE_ERROR));
}
