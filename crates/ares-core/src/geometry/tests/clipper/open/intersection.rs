use super::{coordinates, polygon, polyline};
use crate::geometry::{ClipperError, intersection_open_polylines};

const HI_RANGE: i64 = 0x3fff_ffff_ffff_ffff;

#[test]
fn task22o45_intersection_open_polylines_preserves_order_holes_and_open_paths() {
    let subject = vec![
        polyline(&[(-5, 2), (10, 2)]),
        polyline(&[(10, 2), (25, 2)]),
        polyline(&[(-5, 10), (25, 10)]),
    ];
    let clip = vec![
        polygon(&[(0, 0), (20, 0), (20, 20), (0, 20)]),
        polygon(&[(5, 5), (5, 15), (15, 15), (15, 5)]),
    ];

    let clipped = intersection_open_polylines(&subject, &clip).unwrap();

    assert_eq!(
        coordinates(&clipped),
        vec![
            vec![(5, 10), (0, 10)],
            vec![(20, 10), (15, 10)],
            vec![(20, 2), (10, 2)],
            vec![(10, 2), (0, 2)],
        ]
    );
}

#[test]
fn task22o45_intersection_open_polylines_respects_nonzero_hole() {
    let subject = [polyline(&[(-5, 10), (25, 10)])];
    let clip = [
        polygon(&[(0, 0), (20, 0), (20, 20), (0, 20)]),
        polygon(&[(5, 5), (5, 15), (15, 15), (15, 5)]),
    ];

    let clipped = intersection_open_polylines(&subject, &clip).unwrap();

    assert_eq!(
        coordinates(&clipped),
        vec![vec![(5, 10), (0, 10)], vec![(20, 10), (15, 10)]]
    );
}

#[test]
fn task22o45_intersection_open_polylines_does_not_recombine_touching_subjects() {
    let subject = [polyline(&[(-5, 2), (10, 2)]), polyline(&[(10, 2), (25, 2)])];
    let clip = [polygon(&[(0, 0), (20, 0), (20, 20), (0, 20)])];

    let clipped = intersection_open_polylines(&subject, &clip).unwrap();

    assert_eq!(
        coordinates(&clipped),
        vec![vec![(20, 2), (10, 2)], vec![(10, 2), (0, 2)]]
    );
}

#[test]
fn task22o45_intersection_open_polylines_propagates_range_error() {
    let subject = [polyline(&[(HI_RANGE + 1, 0), (0, 0)])];
    let clip = [polygon(&[(0, -1), (10, -1), (10, 1), (0, 1)])];

    assert_eq!(
        intersection_open_polylines(&subject, &clip),
        Err(ClipperError::CoordinateOutOfRange)
    );
}

#[test]
fn zigzag_splits_at_boundary_crossings() {
    // A zigzag poking out the square's right edge twice: fragment 1 inside
    // (4.5,0)->(10,2.33); fragment 2 (10,3.56)->(2,8)->(4.4,10).
    let subject = [polyline(&[(3, -3), (12, 3), (2, 8), (12, 13)])];
    let clip = [polygon(&[(0, 0), (10, 0), (10, 10), (0, 10)])];
    let clipped = intersection_open_polylines(&subject, &clip).unwrap();
    assert!(
        clipped.len() >= 2,
        "expected multiple fragments, got {}",
        clipped.len()
    );
    assert!(clipped.iter().all(|path| path.points().len() >= 2));
}
