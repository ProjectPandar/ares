mod costs;
mod links;
mod perimeter;

use crate::geometry::{ClipperError, ExPolygon, Point, Polygon};

use super::{
    IntersectionKind, LinkQuality, LinkType, MonotonicRegion, RegionBoundary, SegmentIntersection,
    SegmentedLine, connect_contours, connect_region_neighbors, generate_monotonic_regions,
    insert_phony_outer_pairs, prepare_rectilinear_slice, slice_vertical_lines,
};

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
    let mut slice = prepare_rectilinear_slice(&rectangle(), 0.0, 0.0, 0.0, 3, 10, 40).unwrap();
    connect_contours(&mut slice, false, 0.0);
    let sections = &slice.lines;

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
    let source = prepare_rectilinear_slice(&rectangle(), 0.0, 0.0, 0.0, 2, 10, 80).unwrap();
    let mut disconnected = source.clone();
    connect_contours(&mut disconnected, true, 0.0);
    assert!(
        disconnected
            .lines
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
            .lines
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
fn task22o79_nonpinched_sections_remain_identical() {
    let mut slice = prepare_rectilinear_slice(&rectangle(), 0.0, 0.0, 0.0, 3, 10, 40).unwrap();
    connect_contours(&mut slice, false, 0.0);
    let before = slice.lines.clone();

    insert_phony_outer_pairs(&mut slice.lines);

    assert_eq!(slice.lines, before);
}

#[test]
fn task22o79_disconnected_inner_pair_receives_ordered_phony_outer_pair() {
    let record = |y, kind| SegmentIntersection {
        point: Point::new(50, y),
        contour_index: 0,
        segment_index: y as usize,
        kind,
        previous: None,
        next: None,
    };
    let mut sections = vec![
        SegmentedLine {
            x: 10,
            intersections: Vec::new(),
        },
        SegmentedLine {
            x: 50,
            intersections: vec![
                record(0, IntersectionKind::OuterLow),
                record(20, IntersectionKind::InnerLow),
                record(40, IntersectionKind::InnerHigh),
                record(60, IntersectionKind::InnerLow),
                record(80, IntersectionKind::InnerHigh),
                record(100, IntersectionKind::OuterHigh),
            ],
        },
    ];
    let midpoint = 50;

    insert_phony_outer_pairs(&mut sections);

    assert!(sections[1].intersections.windows(2).any(|pair| {
        pair[0].kind == IntersectionKind::OuterHigh
            && pair[1].kind == IntersectionKind::OuterLow
            && pair[0].point.y() == midpoint
            && pair[1].point.y() == midpoint
            && pair[0].contour_index == usize::MAX
            && pair[1].contour_index == usize::MAX
    }));
}

#[test]
fn task22o80_rectangular_runs_form_one_region_with_source_flip_parity() {
    let prepare = |count| {
        let mut slice =
            prepare_rectilinear_slice(&rectangle(), 0.0, -5.0, -10.0, count, 10, 20).unwrap();
        connect_contours(&mut slice, false, 0.0);
        insert_phony_outer_pairs(&mut slice.lines);
        slice.lines
    };

    let odd = generate_monotonic_regions(&prepare(3));
    let even = generate_monotonic_regions(&prepare(4));

    assert_eq!(odd.len(), 1);
    assert_eq!((odd[0].left.line, odd[0].right.line), (0, 2));
    assert!(odd[0].flips);
    assert_eq!(even.len(), 1);
    assert_eq!((even[0].left.line, even[0].right.line), (0, 3));
    assert!(!even[0].flips);
}

#[test]
fn task22o80_region_generation_is_repeatable_and_does_not_mutate_sections() {
    let mut slice = prepare_rectilinear_slice(&rectangle(), 0.0, -5.0, -10.0, 3, 10, 30).unwrap();
    connect_contours(&mut slice, false, 0.0);
    let lines = &slice.lines;
    let before = lines.clone();

    assert_eq!(
        generate_monotonic_regions(lines),
        generate_monotonic_regions(lines)
    );
    assert_eq!(lines, &before);
}

#[test]
fn task22o82_slice_retains_source_and_indexed_outer_inner_contours() {
    let source = rectangle();
    let slice = prepare_rectilinear_slice(&source, 0.0, -5.0, -10.0, 3, 10, 20).unwrap();

    assert_eq!(slice.source, source);
    assert_eq!(slice.contours.len(), 2);
    assert!(!slice.contours[0].inner);
    assert!(slice.contours[1].inner);
    assert!(
        slice
            .lines
            .iter()
            .flat_map(|line| &line.intersections)
            .all(|item| {
                item.contour_index < slice.contours.len()
                    && item.segment_index
                        < slice.contours[item.contour_index].polygon.points().len()
            })
    );
}

#[test]
fn task22o82_retained_slice_is_repeatable_and_atomic_on_range_error() {
    let source = rectangle();
    let first = prepare_rectilinear_slice(&source, 0.25, -5.0, -10.0, 2, 10, 20).unwrap();
    let second = prepare_rectilinear_slice(&source, 0.25, -5.0, -10.0, 2, 10, 20).unwrap();

    assert_eq!(first, second);
    assert_eq!(source, rectangle());
    assert_eq!(
        prepare_rectilinear_slice(&source, 0.0, 0.0, 0.0, 2, i64::MAX, 1),
        Err(ClipperError::CoordinateOutOfRange)
    );
}

fn intersection(y: i64, kind: IntersectionKind) -> SegmentIntersection {
    SegmentIntersection {
        point: Point::new(0, y),
        contour_index: 0,
        segment_index: 0,
        kind,
        previous: None,
        next: None,
    }
}

fn region(left_line: usize, right_line: usize, low: usize, high: usize) -> MonotonicRegion {
    MonotonicRegion {
        left: RegionBoundary {
            line: left_line,
            low,
            high,
        },
        right: RegionBoundary {
            line: right_line,
            low,
            high,
        },
        flips: true,
        left_neighbors: Vec::new(),
        right_neighbors: Vec::new(),
        lengths: [0.0; 2],
    }
}

#[test]
fn task22o81_region_neighbors_are_sorted_unique_and_symmetric() {
    let mut lines = vec![
        SegmentedLine {
            x: 0,
            intersections: vec![
                intersection(0, IntersectionKind::InnerLow),
                intersection(10, IntersectionKind::InnerHigh),
            ],
        },
        SegmentedLine {
            x: 10,
            intersections: vec![
                intersection(0, IntersectionKind::InnerLow),
                intersection(10, IntersectionKind::InnerHigh),
            ],
        },
    ];
    lines[0].intersections[0].next = Some((0, LinkType::Horizontal, LinkQuality::Valid));
    lines[1].intersections[0].previous = Some((0, LinkType::Horizontal, LinkQuality::Valid));
    let mut regions = vec![region(0, 0, 0, 1), region(1, 1, 0, 1)];

    connect_region_neighbors(&mut regions, &lines);
    connect_region_neighbors(&mut regions, &lines);

    assert_eq!(regions[0].right_neighbors, vec![1]);
    assert_eq!(regions[1].left_neighbors, vec![0]);
    assert!(regions[0].left_neighbors.is_empty());
    assert!(regions[1].right_neighbors.is_empty());
}

#[test]
fn task22o81_regions_without_adjacent_overlap_remain_disconnected() {
    let lines = vec![
        SegmentedLine {
            x: 0,
            intersections: vec![intersection(0, IntersectionKind::InnerLow)],
        },
        SegmentedLine {
            x: 10,
            intersections: vec![intersection(20, IntersectionKind::InnerLow)],
        },
    ];
    let mut regions = vec![region(0, 0, 0, 0), region(1, 1, 0, 0)];

    connect_region_neighbors(&mut regions, &lines);

    assert!(regions.iter().all(|item| item.left_neighbors.is_empty()));
    assert!(regions.iter().all(|item| item.right_neighbors.is_empty()));
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
