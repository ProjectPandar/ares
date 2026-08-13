use crate::geometry::{ClipperError, ExPolygon, Point, Polygon};

use super::{IntersectionKind, LinkQuality, LinkType, connect_contours, slice_vertical_lines};

fn rectangle() -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(100, 0),
            Point::new(100, 80),
            Point::new(0, 80),
        ]),
        Vec::new(),
    )
}

#[test]
fn task22o77_rectangle_vertical_sections_preserve_source_kinds_and_order() {
    let rectangle = rectangle();
    let before = rectangle.clone();
    let sections = slice_vertical_lines(&rectangle, 0.0, 0.0, 0.0, 3, 10, 40).unwrap();

    assert_eq!(rectangle, before);
    assert_eq!(sections.len(), 3);
    assert_eq!(
        sections
            .iter()
            .map(|section| (
                section.x,
                section
                    .intersections
                    .iter()
                    .map(|intersection| (intersection.point.y(), intersection.kind))
                    .collect::<Vec<_>>()
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                10,
                vec![
                    (0, IntersectionKind::OuterLow),
                    (80, IntersectionKind::OuterHigh)
                ]
            ),
            (
                50,
                vec![
                    (0, IntersectionKind::OuterLow),
                    (80, IntersectionKind::OuterHigh)
                ]
            ),
            (
                90,
                vec![
                    (0, IntersectionKind::OuterLow),
                    (80, IntersectionKind::OuterHigh)
                ]
            ),
        ]
    );
}

#[test]
fn task22o77_hole_and_inner_offset_preserve_outer_before_inner_identity() {
    let donut = ExPolygon::new(
        rectangle().contour().clone(),
        vec![Polygon::new(vec![
            Point::new(30, 20),
            Point::new(30, 60),
            Point::new(70, 60),
            Point::new(70, 20),
        ])],
    );
    let with_hole = slice_vertical_lines(&donut, 0.0, 0.0, 0.0, 1, 50, 1).unwrap();
    assert_eq!(
        with_hole[0]
            .intersections
            .iter()
            .map(|intersection| (intersection.point.y(), intersection.kind))
            .collect::<Vec<_>>(),
        vec![
            (0, IntersectionKind::OuterLow),
            (20, IntersectionKind::OuterHigh),
            (60, IntersectionKind::OuterLow),
            (80, IntersectionKind::OuterHigh),
        ]
    );

    let offset = slice_vertical_lines(&rectangle(), 0.0, -5.0, -10.0, 1, 50, 1).unwrap();
    assert!(offset[0].intersections.iter().any(|intersection| matches!(
        intersection.kind,
        IntersectionKind::InnerLow | IntersectionKind::InnerHigh
    )));
}

#[test]
fn task22o78_rectangle_intersections_link_horizontally_and_symmetrically() {
    let mut sections = slice_vertical_lines(&rectangle(), 0.0, 0.0, 0.0, 3, 10, 40).unwrap();
    connect_contours(&mut sections, false, 0.0);

    assert_eq!(
        sections[1].intersections[0].previous,
        Some((0, LinkType::Horizontal, LinkQuality::Valid))
    );
    assert_eq!(
        sections[1].intersections[0].next,
        Some((0, LinkType::Horizontal, LinkQuality::Valid))
    );
    assert_eq!(
        sections[0].intersections[0].next,
        Some((0, LinkType::Horizontal, LinkQuality::Valid))
    );
    assert_eq!(
        sections[2].intersections[0].previous,
        Some((0, LinkType::Horizontal, LinkQuality::Valid))
    );
}

#[test]
fn task22o78_dont_connect_and_max_length_change_only_link_quality() {
    let source = slice_vertical_lines(&rectangle(), 0.0, 0.0, 0.0, 2, 10, 80).unwrap();
    let mut disconnected = source.clone();
    connect_contours(&mut disconnected, true, 0.0);
    assert!(
        disconnected
            .iter()
            .flat_map(|line| &line.intersections)
            .all(|item| {
                item.previous
                    .is_none_or(|link| link.2 == LinkQuality::TooLong)
                    && item.next.is_none_or(|link| link.2 == LinkQuality::TooLong)
            })
    );

    let mut limited = source;
    connect_contours(&mut limited, false, 10.0);
    assert!(
        limited
            .iter()
            .flat_map(|line| &line.intersections)
            .any(|item| {
                item.previous
                    .is_some_and(|link| link.2 == LinkQuality::TooLong)
                    || item.next.is_some_and(|link| link.2 == LinkQuality::TooLong)
            })
    );
}

#[test]
fn task22o77_rational_rounding_rotation_and_range_error_are_deterministic() {
    let triangle = ExPolygon::new(
        Polygon::new(vec![Point::new(0, 0), Point::new(9, 5), Point::new(0, 10)]),
        Vec::new(),
    );
    let first = slice_vertical_lines(&triangle, 0.0, 0.0, 0.0, 1, 4, 1).unwrap();
    let second = slice_vertical_lines(&triangle, 0.0, 0.0, 0.0, 1, 4, 1).unwrap();
    assert_eq!(first, second);
    assert_eq!(
        first[0]
            .intersections
            .iter()
            .map(|intersection| intersection.point.y())
            .collect::<Vec<_>>(),
        vec![2, 8]
    );
    assert_ne!(
        slice_vertical_lines(&triangle, 0.25, 0.0, 0.0, 1, 4, 1).unwrap(),
        first
    );
    assert_eq!(
        slice_vertical_lines(&triangle, 0.0, 0.0, 0.0, 2, i64::MAX, 1),
        Err(ClipperError::CoordinateOutOfRange)
    );
}
