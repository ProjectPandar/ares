use crate::geometry::medial_axis::annotate::{
    CellCategory, merge_cell_category, vertex_equal_to_point,
};
use crate::geometry::{
    Line, Point,
    medial_axis::{annotate, diagram},
};

#[test]
fn task22o13_voronoi_point_equality_uses_the_source_64_ulp_window() {
    let point = 12.0_f64;
    let within = f64::from_bits(point.to_bits() + 64);
    let outside = f64::from_bits(point.to_bits() + 65);
    assert!(vertex_equal_to_point(point, within));
    assert!(vertex_equal_to_point(-point, -within));
    assert!(!vertex_equal_to_point(point, outside));
    assert!(!vertex_equal_to_point(-point, -outside));
    assert!(vertex_equal_to_point(-0.0, 0.0));
    assert!(vertex_equal_to_point(
        -f64::from_bits(32),
        f64::from_bits(32)
    ));
    assert!(!vertex_equal_to_point(
        -f64::from_bits(33),
        f64::from_bits(33)
    ));
}

#[test]
fn task22o13_boundary_annotation_overrides_and_remains_sticky() {
    for old in [
        CellCategory::Unknown,
        CellCategory::Inside,
        CellCategory::Outside,
        CellCategory::Boundary,
    ] {
        assert_eq!(
            merge_cell_category(old, CellCategory::Boundary),
            CellCategory::Boundary
        );
    }
    for new in [
        CellCategory::Unknown,
        CellCategory::Inside,
        CellCategory::Outside,
        CellCategory::Boundary,
    ] {
        assert_eq!(
            merge_cell_category(CellCategory::Boundary, new),
            CellCategory::Boundary
        );
    }
}

#[test]
fn task22o13_conflicting_inside_outside_annotations_become_boundary() {
    assert_eq!(
        merge_cell_category(CellCategory::Inside, CellCategory::Outside),
        CellCategory::Boundary
    );
    assert_eq!(
        merge_cell_category(CellCategory::Outside, CellCategory::Inside),
        CellCategory::Boundary
    );
}

fn annotation_snapshot(lines: &[Line]) -> (Vec<String>, Vec<String>, Vec<String>) {
    let vd = diagram::build(lines).unwrap();
    let annotations = annotate::annotate(&vd, lines).unwrap();
    (
        annotations
            .vertices
            .iter()
            .enumerate()
            .map(|(id, category)| format!("vertex={id} category={category:?}"))
            .collect(),
        annotations
            .edges
            .iter()
            .enumerate()
            .map(|(id, category)| format!("edge={id} category={category:?}"))
            .collect(),
        annotations
            .cells
            .iter()
            .enumerate()
            .map(|(id, category)| format!("cell={id} category={category:?}"))
            .collect(),
    )
}

#[test]
fn task22o13_contour_and_point_queue_annotation_is_literal() {
    let lines = [
        Line::new(Point::new(0, 0), Point::new(20, 0)),
        Line::new(Point::new(20, 0), Point::new(20, 10)),
        Line::new(Point::new(20, 10), Point::new(0, 10)),
        Line::new(Point::new(0, 10), Point::new(0, 0)),
    ];
    let snapshot = annotation_snapshot(&lines);
    assert_eq!(
        snapshot.0,
        [
            "vertex=0 category=OnContour",
            "vertex=1 category=OnContour",
            "vertex=2 category=Inside",
            "vertex=3 category=OnContour",
            "vertex=4 category=Inside",
            "vertex=5 category=OnContour",
        ]
    );
    assert_eq!(
        snapshot.1,
        [
            "edge=0 category=Outside",
            "edge=1 category=ToContour",
            "edge=2 category=Outside",
            "edge=3 category=ToContour",
            "edge=4 category=ToContour",
            "edge=5 category=Outside",
            "edge=6 category=ToContour",
            "edge=7 category=Inside",
            "edge=8 category=Outside",
            "edge=9 category=ToContour",
            "edge=10 category=ToContour",
            "edge=11 category=Inside",
            "edge=12 category=Inside",
            "edge=13 category=Inside",
            "edge=14 category=ToContour",
            "edge=15 category=Outside",
            "edge=16 category=ToContour",
            "edge=17 category=Outside",
            "edge=18 category=ToContour",
            "edge=19 category=Inside",
            "edge=20 category=Inside",
            "edge=21 category=ToContour",
            "edge=22 category=ToContour",
            "edge=23 category=Outside",
            "edge=24 category=ToContour",
            "edge=25 category=Outside",
        ]
    );
    assert_eq!(
        snapshot.2,
        [
            "cell=0 category=Outside",
            "cell=1 category=Boundary",
            "cell=2 category=Outside",
            "cell=3 category=Boundary",
            "cell=4 category=Boundary",
            "cell=5 category=Outside",
            "cell=6 category=Boundary",
            "cell=7 category=Outside",
        ]
    );
}

#[test]
fn task22o13_hole_annotation_is_literal() {
    let lines = [
        Line::new(Point::new(0, 0), Point::new(40, 0)),
        Line::new(Point::new(40, 0), Point::new(40, 40)),
        Line::new(Point::new(40, 40), Point::new(0, 40)),
        Line::new(Point::new(0, 40), Point::new(0, 0)),
        Line::new(Point::new(10, 10), Point::new(10, 30)),
        Line::new(Point::new(10, 30), Point::new(30, 30)),
        Line::new(Point::new(30, 30), Point::new(30, 10)),
        Line::new(Point::new(30, 10), Point::new(10, 10)),
    ];
    let snapshot = annotation_snapshot(&lines);
    assert_eq!(
        snapshot.0,
        [
            "vertex=0 category=OnContour",
            "vertex=1 category=OnContour",
            "vertex=2 category=Inside",
            "vertex=3 category=Inside",
            "vertex=4 category=OnContour",
            "vertex=5 category=OnContour",
            "vertex=6 category=Inside",
            "vertex=7 category=Inside",
            "vertex=8 category=Inside",
            "vertex=9 category=Inside",
            "vertex=10 category=OnContour",
            "vertex=11 category=Outside",
            "vertex=12 category=OnContour",
            "vertex=13 category=Inside",
            "vertex=14 category=Inside",
            "vertex=15 category=OnContour",
            "vertex=16 category=Inside",
            "vertex=17 category=Inside",
            "vertex=18 category=Inside",
            "vertex=19 category=Inside",
            "vertex=20 category=OnContour",
        ]
    );
    assert_eq!(
        snapshot.1,
        [
            "edge=0 category=Outside",
            "edge=1 category=ToContour",
            "edge=2 category=Outside",
            "edge=3 category=ToContour",
            "edge=4 category=ToContour",
            "edge=5 category=Outside",
            "edge=6 category=ToContour",
            "edge=7 category=Inside",
            "edge=8 category=Outside",
            "edge=9 category=ToContour",
            "edge=10 category=ToContour",
            "edge=11 category=Inside",
            "edge=12 category=Inside",
            "edge=13 category=Inside",
            "edge=14 category=Inside",
            "edge=15 category=ToContour",
            "edge=16 category=Inside",
            "edge=17 category=Inside",
            "edge=18 category=Inside",
            "edge=19 category=ToContour",
            "edge=20 category=Inside",
            "edge=21 category=Inside",
            "edge=22 category=ToContour",
            "edge=23 category=Inside",
            "edge=24 category=ToContour",
            "edge=25 category=Outside",
            "edge=26 category=Inside",
            "edge=27 category=ToContour",
            "edge=28 category=ToContour",
            "edge=29 category=Outside",
            "edge=30 category=Inside",
            "edge=31 category=Inside",
            "edge=32 category=Inside",
            "edge=33 category=Inside",
            "edge=34 category=Inside",
            "edge=35 category=Inside",
            "edge=36 category=Inside",
            "edge=37 category=Inside",
            "edge=38 category=ToContour",
            "edge=39 category=Inside",
            "edge=40 category=ToContour",
            "edge=41 category=Inside",
            "edge=42 category=ToContour",
            "edge=43 category=Outside",
            "edge=44 category=Outside",
            "edge=45 category=ToContour",
            "edge=46 category=ToContour",
            "edge=47 category=Inside",
            "edge=48 category=ToContour",
            "edge=49 category=Inside",
            "edge=50 category=Inside",
            "edge=51 category=Inside",
            "edge=52 category=Inside",
            "edge=53 category=Inside",
            "edge=54 category=ToContour",
            "edge=55 category=Outside",
            "edge=56 category=ToContour",
            "edge=57 category=Outside",
            "edge=58 category=ToContour",
            "edge=59 category=Inside",
            "edge=60 category=Inside",
            "edge=61 category=Inside",
            "edge=62 category=Inside",
            "edge=63 category=Inside",
            "edge=64 category=Inside",
            "edge=65 category=Inside",
            "edge=66 category=Inside",
            "edge=67 category=ToContour",
            "edge=68 category=ToContour",
            "edge=69 category=Outside",
            "edge=70 category=ToContour",
            "edge=71 category=Outside",
        ]
    );
    assert_eq!(
        snapshot.2,
        [
            "cell=0 category=Outside",
            "cell=1 category=Boundary",
            "cell=2 category=Outside",
            "cell=3 category=Boundary",
            "cell=4 category=Boundary",
            "cell=5 category=Inside",
            "cell=6 category=Boundary",
            "cell=7 category=Inside",
            "cell=8 category=Boundary",
            "cell=9 category=Boundary",
            "cell=10 category=Inside",
            "cell=11 category=Boundary",
            "cell=12 category=Inside",
            "cell=13 category=Outside",
            "cell=14 category=Boundary",
            "cell=15 category=Outside",
        ]
    );
}
