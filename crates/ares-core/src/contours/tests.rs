use super::*;
use crate::{LayerSlice, Point2, Segment2, SliceError};

#[test]
fn stitches_unordered_square_segments_into_ccw_contour() {
    let slices = [LayerSlice::new(
        0,
        0.2,
        vec![
            Segment2::new(Point2::new(1.0, 0.0), Point2::new(1.0, 1.0)),
            Segment2::new(Point2::new(0.0, 1.0), Point2::new(0.0, 0.0)),
            Segment2::new(Point2::new(1.0, 1.0), Point2::new(0.0, 1.0)),
            Segment2::new(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)),
        ],
    )];

    let contours = stitch_layer_slices(&slices).unwrap();

    assert_eq!(contours.len(), 1);
    assert_eq!(contours[0].layer_id(), 0);
    assert_eq!(contours[0].print_z(), 0.2);
    assert_eq!(contours[0].contours().len(), 1);
    assert_eq!(
        contours[0].contours()[0].points(),
        &[
            Point2::new(0.0, 0.0),
            Point2::new(1.0, 0.0),
            Point2::new(1.0, 1.0),
            Point2::new(0.0, 1.0),
        ]
    );
}

#[test]
fn orders_multiple_contours_deterministically() {
    let slices = [LayerSlice::new(
        0,
        0.2,
        vec![
            Segment2::new(Point2::new(10.0, 10.0), Point2::new(11.0, 10.0)),
            Segment2::new(Point2::new(11.0, 10.0), Point2::new(11.0, 11.0)),
            Segment2::new(Point2::new(11.0, 11.0), Point2::new(10.0, 11.0)),
            Segment2::new(Point2::new(10.0, 11.0), Point2::new(10.0, 10.0)),
            Segment2::new(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)),
            Segment2::new(Point2::new(1.0, 0.0), Point2::new(1.0, 1.0)),
            Segment2::new(Point2::new(1.0, 1.0), Point2::new(0.0, 1.0)),
            Segment2::new(Point2::new(0.0, 1.0), Point2::new(0.0, 0.0)),
        ],
    )];

    let contours = stitch_layer_slices(&slices).unwrap();

    assert_eq!(
        contours[0].contours()[0].points(),
        &[
            Point2::new(0.0, 0.0),
            Point2::new(1.0, 0.0),
            Point2::new(1.0, 1.0),
            Point2::new(0.0, 1.0),
        ]
    );
    assert_eq!(
        contours[0].contours()[1].points(),
        &[
            Point2::new(10.0, 10.0),
            Point2::new(11.0, 10.0),
            Point2::new(11.0, 11.0),
            Point2::new(10.0, 11.0),
        ]
    );
}

#[test]
fn normalizes_clockwise_contours_to_counter_clockwise() {
    let slices = [LayerSlice::new(
        0,
        0.2,
        vec![
            Segment2::new(Point2::new(0.0, 0.0), Point2::new(0.0, 1.0)),
            Segment2::new(Point2::new(0.0, 1.0), Point2::new(1.0, 1.0)),
            Segment2::new(Point2::new(1.0, 1.0), Point2::new(1.0, 0.0)),
            Segment2::new(Point2::new(1.0, 0.0), Point2::new(0.0, 0.0)),
        ],
    )];

    let contours = stitch_layer_slices(&slices).unwrap();

    assert_eq!(
        contours[0].contours()[0].points(),
        &[
            Point2::new(0.0, 0.0),
            Point2::new(1.0, 0.0),
            Point2::new(1.0, 1.0),
            Point2::new(0.0, 1.0),
        ]
    );
}

#[test]
fn preserves_empty_layer_contours() {
    let slices = [LayerSlice::new(7, 1.4, Vec::new())];

    let contours = stitch_layer_slices(&slices).unwrap();

    assert_eq!(contours.len(), 1);
    assert_eq!(contours[0].layer_id(), 7);
    assert!(contours[0].contours().is_empty());
}

#[test]
fn rejects_open_chains() {
    let slices = [LayerSlice::new(
        0,
        0.2,
        vec![Segment2::new(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0))],
    )];

    let err = stitch_layer_slices(&slices).unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
}

#[test]
fn rejects_branching_graphs() {
    let slices = [LayerSlice::new(
        0,
        0.2,
        vec![
            Segment2::new(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)),
            Segment2::new(Point2::new(1.0, 0.0), Point2::new(1.0, 1.0)),
            Segment2::new(Point2::new(1.0, 0.0), Point2::new(2.0, 0.0)),
        ],
    )];

    let err = stitch_layer_slices(&slices).unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
}

#[test]
fn rejects_duplicate_two_edge_multigraphs() {
    let slices = [LayerSlice::new(
        0,
        0.2,
        vec![
            Segment2::new(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)),
            Segment2::new(Point2::new(1.0, 0.0), Point2::new(0.0, 0.0)),
        ],
    )];

    let err = stitch_layer_slices(&slices).unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
}

#[test]
fn rejects_duplicate_segment_in_closed_loop() {
    let slices = [LayerSlice::new(
        0,
        0.2,
        vec![
            Segment2::new(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)),
            Segment2::new(Point2::new(1.0, 0.0), Point2::new(1.0, 1.0)),
            Segment2::new(Point2::new(1.0, 1.0), Point2::new(0.0, 1.0)),
            Segment2::new(Point2::new(0.0, 1.0), Point2::new(0.0, 0.0)),
            Segment2::new(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)),
        ],
    )];

    let err = stitch_layer_slices(&slices).unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
}

#[test]
fn orders_layer_contours_by_layer_id() {
    let slices = [
        LayerSlice::new(
            1,
            0.4,
            vec![
                Segment2::new(Point2::new(10.0, 10.0), Point2::new(11.0, 10.0)),
                Segment2::new(Point2::new(11.0, 10.0), Point2::new(11.0, 11.0)),
                Segment2::new(Point2::new(11.0, 11.0), Point2::new(10.0, 11.0)),
                Segment2::new(Point2::new(10.0, 11.0), Point2::new(10.0, 10.0)),
            ],
        ),
        LayerSlice::new(
            0,
            0.2,
            vec![
                Segment2::new(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)),
                Segment2::new(Point2::new(1.0, 0.0), Point2::new(1.0, 1.0)),
                Segment2::new(Point2::new(1.0, 1.0), Point2::new(0.0, 1.0)),
                Segment2::new(Point2::new(0.0, 1.0), Point2::new(0.0, 0.0)),
            ],
        ),
    ];

    let contours = stitch_layer_slices(&slices).unwrap();

    assert_eq!(contours[0].layer_id(), 0);
    assert_eq!(contours[0].print_z(), 0.2);
    assert_eq!(
        contours[0].contours()[0].points(),
        &[
            Point2::new(0.0, 0.0),
            Point2::new(1.0, 0.0),
            Point2::new(1.0, 1.0),
            Point2::new(0.0, 1.0),
        ]
    );
    assert_eq!(contours[1].layer_id(), 1);
    assert_eq!(contours[1].print_z(), 0.4);
    assert_eq!(
        contours[1].contours()[0].points(),
        &[
            Point2::new(10.0, 10.0),
            Point2::new(11.0, 10.0),
            Point2::new(11.0, 11.0),
            Point2::new(10.0, 11.0),
        ]
    );
}
