use crate::geometry::{Point, Polygon};

use super::super::prepare_rectilinear_slice;
use super::super::segments::{OffsetContour, RectilinearSlice};
use super::super::{
    IntersectionKind, LinkQuality, LinkType, SegmentIntersection, SegmentedLine, connect_contours,
};
use super::rectangle;

fn intersection(y: i64, segment_index: usize, kind: IntersectionKind) -> SegmentIntersection {
    SegmentIntersection {
        point: Point::new(0, y),
        position: super::super::segments::RationalPosition::integer(y),
        contour_index: 0,
        segment_index,
        kind,
        previous: None,
        next: None,
    }
}

#[test]
fn task22o84_link_selection_uses_directed_contour_wraparound_not_absolute_index() {
    let contour = Polygon::new(vec![
        Point::new(0, 0),
        Point::new(100, 0),
        Point::new(100, 100),
        Point::new(0, 100),
    ]);
    let side = |x| SegmentedLine {
        x,
        intersections: vec![
            intersection(0, 3, IntersectionKind::OuterLow),
            intersection(10, 1, IntersectionKind::OuterLow),
            intersection(20, 2, IntersectionKind::OuterHigh),
        ],
    };
    let mut slice = RectilinearSlice {
        source: rectangle(),
        contours: vec![OffsetContour {
            polygon: contour,
            inner: false,
        }],
        lines: vec![
            side(-10),
            SegmentedLine {
                x: 0,
                intersections: vec![
                    intersection(0, 0, IntersectionKind::OuterLow),
                    intersection(20, 2, IntersectionKind::OuterHigh),
                ],
            },
            side(10),
        ],
    };

    connect_contours(&mut slice, false, 0.0);

    assert_eq!(
        slice.lines[1].intersections[0].previous,
        Some((0, LinkType::Horizontal, LinkQuality::Valid))
    );
    assert_eq!(
        slice.lines[1].intersections[0].next,
        Some((1, LinkType::Horizontal, LinkQuality::Valid))
    );
}

#[test]
fn task22o84_contour_length_gate_is_strict_and_uses_perimeter_arc() {
    let source = prepare_rectilinear_slice(&rectangle(), 0.0, 0.0, 0.0, 2, 10, 80).unwrap();
    let mut equal = source.clone();
    connect_contours(&mut equal, false, 80.0);
    assert_eq!(
        equal.lines[1].intersections[0].previous,
        Some((0, LinkType::Horizontal, LinkQuality::Valid))
    );

    let mut below = source;
    connect_contours(&mut below, false, 79.0);
    assert_eq!(
        below.lines[1].intersections[0].previous,
        Some((0, LinkType::Horizontal, LinkQuality::TooLong))
    );
}
