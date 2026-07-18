use crate::{
    geometry::{Point, Polygon},
    mesh_slicer::{LoopedLayer, SlicingMode, apply_slicing_mode},
};

const A: &[(i64, i64)] = &[(0, 0), (4, 0), (0, 3)];
const B: &[(i64, i64)] = &[(10, 0), (10, 2), (14, 2), (14, 0)];
const B_POSITIVE: &[(i64, i64)] = &[(14, 0), (14, 2), (10, 2), (10, 0)];
const C: &[(i64, i64)] = &[(20, 0), (20, 5), (26, 5), (26, 0)];
const C_POSITIVE: &[(i64, i64)] = &[(26, 0), (26, 5), (20, 5), (20, 0)];

#[test]
fn task22e_regular_and_even_odd_are_exact_raw_identity_modes() {
    let original = layer(&[A, B, C]);

    for mode in [SlicingMode::Regular, SlicingMode::EvenOdd] {
        let mut actual = original.clone();
        apply_slicing_mode(&mut actual, mode);
        assert_eq!(actual, original);
    }
}

#[test]
fn task22e_positive_reverses_only_clockwise_complete_point_vectors() {
    let zero = &[(30, 0), (31, 0), (32, 0)];
    let mut actual = layer(&[A, B, C, zero]);

    apply_slicing_mode(&mut actual, SlicingMode::Positive);

    assert_eq!(actual, layer(&[A, B_POSITIVE, C_POSITIVE, zero]));
}

#[test]
fn task22e_positive_largest_selects_strict_greatest_absolute_area() {
    let mut actual = layer(&[A, B, C]);

    apply_slicing_mode(&mut actual, SlicingMode::PositiveLargestContour);

    assert_eq!(actual, layer(&[C_POSITIVE]));
}

#[test]
fn task22e_positive_largest_equal_area_tie_keeps_and_reverses_first() {
    let first = &[(0, 0), (0, 2), (2, 2), (2, 0)];
    let second = &[(10, 0), (12, 0), (12, 2), (10, 2)];
    let expected = &[(2, 0), (2, 2), (0, 2), (0, 0)];
    let mut actual = layer(&[first, second]);

    apply_slicing_mode(&mut actual, SlicingMode::PositiveLargestContour);

    assert_eq!(actual, layer(&[expected]));
}

#[test]
fn task22e_positive_largest_preserves_empty_and_normalizes_single_inputs() {
    let mut empty = LoopedLayer::default();
    apply_slicing_mode(&mut empty, SlicingMode::PositiveLargestContour);
    assert!(empty.polygons().is_empty());

    let mut ccw = layer(&[A]);
    let ccw_before = ccw.clone();
    apply_slicing_mode(&mut ccw, SlicingMode::PositiveLargestContour);
    assert_eq!(ccw, ccw_before);

    let mut cw = layer(&[B]);
    apply_slicing_mode(&mut cw, SlicingMode::PositiveLargestContour);
    assert_eq!(cw, layer(&[B_POSITIVE]));
}

#[test]
#[should_panic(expected = "positive-largest contour requires a nonzero-area polygon")]
fn task22e_positive_largest_nonempty_all_zero_area_is_an_internal_invariant() {
    let mut actual = layer(&[&[(0, 0), (1, 0), (2, 0)]]);
    apply_slicing_mode(&mut actual, SlicingMode::PositiveLargestContour);
}

fn layer(polygons: &[&[(i64, i64)]]) -> LoopedLayer {
    let mut layer = LoopedLayer::default();
    layer.polygons_mut().extend(
        polygons
            .iter()
            .map(|points| Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())),
    );
    layer
}
