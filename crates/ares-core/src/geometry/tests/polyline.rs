use crate::geometry::{Point, Polyline, ThickLine, ThickPolyline, to_thick_polylines};

#[test]
fn task22o6_polyline_preserves_open_point_order_and_endpoints() {
    let polyline = Polyline::new(vec![
        Point::new(5, 8),
        Point::new(-3, 13),
        Point::new(21, -1),
    ]);

    assert_eq!(
        polyline.points(),
        &[Point::new(5, 8), Point::new(-3, 13), Point::new(21, -1)]
    );
    assert_eq!(polyline.front(), Some(Point::new(5, 8)));
    assert_eq!(polyline.back(), Some(Point::new(21, -1)));
}

#[test]
fn task22o6_polyline_reversal_is_exact_and_does_not_close() {
    let mut polyline = Polyline::new(vec![Point::new(1, 2), Point::new(3, 4), Point::new(5, 6)]);

    polyline.reverse();

    assert_eq!(
        polyline.into_points(),
        vec![Point::new(5, 6), Point::new(3, 4), Point::new(1, 2)]
    );
}

#[test]
fn task22o6_polyline_validity_requires_exactly_two_or_more_points() {
    assert!(!Polyline::new(Vec::new()).is_valid());
    assert!(!Polyline::new(vec![Point::new(0, 0)]).is_valid());
    assert!(Polyline::new(vec![Point::new(0, 0), Point::new(0, 0)]).is_valid());
}

#[test]
fn task22o12_thick_polyline_default_reverse_and_clear_match_source() {
    assert_eq!(ThickPolyline::default().endpoints, (false, false));
    let mut polyline = ThickPolyline {
        points: vec![Point::new(1, 1), Point::new(2, 2), Point::new(3, 3)],
        width: vec![10.0, 11.0, 20.0, 21.0],
        endpoints: (true, false),
    };
    polyline.reverse();
    assert_eq!(
        polyline,
        ThickPolyline {
            points: vec![Point::new(3, 3), Point::new(2, 2), Point::new(1, 1)],
            width: vec![21.0, 20.0, 11.0, 10.0],
            endpoints: (false, true),
        }
    );
    polyline.clear();
    assert!(polyline.points.is_empty());
    assert!(polyline.width.is_empty());
    assert_eq!(polyline.endpoints, (false, true));
}

#[test]
fn task22o12_thicklines_pair_each_segment_with_two_ordered_widths() {
    let polyline = ThickPolyline {
        points: vec![Point::new(0, 0), Point::new(5, 1), Point::new(8, 9)],
        width: vec![1.0, 2.0, 3.0, 4.0],
        endpoints: (false, false),
    };
    assert_eq!(
        polyline.thicklines(),
        vec![
            ThickLine::with_widths(Point::new(0, 0), Point::new(5, 1), 1.0, 2.0),
            ThickLine::with_widths(Point::new(5, 1), Point::new(8, 9), 3.0, 4.0),
        ]
    );
    assert!(ThickPolyline::default().thicklines().is_empty());
}

#[test]
fn task22o12_closed_thick_polyline_rotation_moves_one_point_and_two_widths() {
    let source = ThickPolyline {
        points: vec![
            Point::new(0, 0),
            Point::new(4, 0),
            Point::new(4, 4),
            Point::new(0, 0),
        ],
        width: vec![10.0, 11.0, 20.0, 21.0, 30.0, 10.0],
        endpoints: (true, false),
    };
    for index in [0, 3] {
        let mut unchanged = source.clone();
        unchanged.start_at_index(index);
        assert_eq!(unchanged, source);
    }
    let mut rotated = source;
    rotated.start_at_index(1);
    assert_eq!(
        rotated,
        ThickPolyline {
            points: vec![
                Point::new(4, 0),
                Point::new(4, 4),
                Point::new(0, 0),
                Point::new(4, 0),
            ],
            width: vec![20.0, 21.0, 30.0, 10.0, 10.0, 11.0],
            endpoints: (true, false),
        }
    );
}

#[test]
fn task22o201_thick_polyline_clip_end_interpolates_points_and_retains_width_payload() {
    let mut polyline = ThickPolyline {
        points: vec![Point::new(0, 0), Point::new(10, 0), Point::new(10, 10)],
        width: vec![1.0, 2.0, 3.0, 4.0],
        endpoints: (false, false),
    };

    polyline.clip_end(5.0);

    assert_eq!(
        polyline.points,
        [Point::new(0, 0), Point::new(10, 0), Point::new(10, 5)]
    );
    assert_eq!(polyline.width, [1.0, 2.0, 3.0, 4.0]);
}
#[test]
fn task22o13_thick_polyline_length_sums_adjacent_stored_segments() {
    let polyline = ThickPolyline {
        points: vec![Point::new(0, 0), Point::new(3, 4), Point::new(3, 12)],
        width: vec![1.0, 2.0, 3.0, 4.0],
        endpoints: (true, true),
    };
    assert_eq!(polyline.length(), 13.0);
}

#[test]
fn task22o12_to_thick_polylines_moves_order_and_assigns_two_widths_per_segment() {
    let output = to_thick_polylines(
        vec![
            Polyline::new(vec![Point::new(0, 0), Point::new(1, 0)]),
            Polyline::new(vec![Point::new(5, 5), Point::new(6, 5), Point::new(7, 8)]),
        ],
        2.5,
    );
    assert_eq!(
        output,
        vec![
            ThickPolyline {
                points: vec![Point::new(0, 0), Point::new(1, 0)],
                width: vec![2.5, 2.5],
                endpoints: (false, false),
            },
            ThickPolyline {
                points: vec![Point::new(5, 5), Point::new(6, 5), Point::new(7, 8)],
                width: vec![2.5; 4],
                endpoints: (false, false),
            },
        ]
    );
}
