use crate::geometry::{Point, Polygon};

use super::super::types::{LoopBuckets, PerimeterGeneratorLoop};
use super::nest;

fn square(min: i64, max: i64) -> Polygon {
    Polygon::new(vec![
        Point::new(min, min),
        Point::new(max, min),
        Point::new(max, max),
        Point::new(min, max),
    ])
}

fn loop_(polygon: Polygon, depth: u16, contour: bool) -> PerimeterGeneratorLoop {
    PerimeterGeneratorLoop {
        polygon,
        is_contour: contour,
        is_smaller_width_perimeter: false,
        depth,
        children: Vec::new(),
    }
}

#[test]
fn task22o4_holes_choose_first_deeper_hole_before_any_contour() {
    let buckets = LoopBuckets {
        contours: vec![vec![loop_(square(0, 100), 0, true)], Vec::new()],
        holes: vec![
            vec![loop_(square(20, 21), 0, false)],
            vec![
                loop_(square(10, 40), 1, false),
                loop_(square(5, 50), 1, false),
            ],
        ],
    };
    let nested = nest(buckets);
    let root_children = &nested.roots[0].children;
    assert_eq!(root_children.len(), 2);
    assert_eq!(root_children[0].polygon.points()[0], Point::new(10, 10));
    assert_eq!(root_children[0].children.len(), 1);
    assert_eq!(
        root_children[0].children[0].polygon.points()[0],
        Point::new(20, 20)
    );
    assert_eq!(root_children[1].polygon.points()[0], Point::new(5, 5));
}

#[test]
fn task22o4_hole_parent_depth_search_starts_at_the_next_depth() {
    let buckets = LoopBuckets {
        contours: vec![vec![loop_(square(0, 100), 0, true)], Vec::new(), Vec::new()],
        holes: vec![
            vec![loop_(square(20, 21), 0, false)],
            vec![loop_(square(10, 40), 1, false)],
            vec![loop_(square(5, 50), 2, false)],
        ],
    };
    let nested = nest(buckets);
    let depth_two = &nested.roots[0].children[0];
    let depth_one = &depth_two.children[0];
    assert_eq!(depth_two.depth, 2);
    assert_eq!(depth_one.depth, 1);
    assert_eq!(depth_one.children[0].depth, 0);
}

#[test]
fn task22o4_contour_remove_retries_shifted_item_and_keeps_root_order() {
    let buckets = LoopBuckets {
        contours: vec![
            vec![
                loop_(square(0, 100), 0, true),
                loop_(square(200, 300), 0, true),
            ],
            vec![
                loop_(square(10, 20), 1, true),
                loop_(square(30, 40), 1, true),
            ],
        ],
        holes: vec![Vec::new(), Vec::new()],
    };
    let nested = nest(buckets);
    assert_eq!(nested.roots.len(), 2);
    assert_eq!(nested.roots[0].children.len(), 2);
    assert_eq!(
        nested.roots[0].children[0].polygon.points()[0],
        Point::new(10, 10)
    );
    assert_eq!(
        nested.roots[0].children[1].polygon.points()[0],
        Point::new(30, 30)
    );
    assert_eq!(nested.roots[1].polygon.points()[0], Point::new(200, 200));
}

#[test]
fn task22o4_uses_boundary_inclusive_child_first_point_and_preserves_orphans() {
    let boundary_child = Polygon::new(vec![
        Point::new(0, 5),
        Point::new(-20, 5),
        Point::new(-20, 6),
    ]);
    let buckets = LoopBuckets {
        contours: vec![
            vec![loop_(square(0, 10), 0, true)],
            vec![
                loop_(boundary_child, 1, true),
                loop_(square(30, 40), 1, true),
            ],
        ],
        holes: vec![vec![loop_(square(50, 60), 0, false)], Vec::new()],
    };
    let nested = nest(buckets);
    assert_eq!(nested.roots[0].children.len(), 1);
    assert_eq!(
        nested.roots[0].children[0].polygon.points()[0],
        Point::new(0, 5)
    );
    assert_eq!(nested.contours[1].len(), 1);
    assert_eq!(nested.holes[0].len(), 1);
}

#[test]
fn task22o4_searches_deep_contours_first_and_first_candidate_by_index() {
    let buckets = LoopBuckets {
        contours: vec![
            vec![loop_(square(0, 100), 0, true)],
            vec![
                loop_(square(10, 90), 1, true),
                loop_(square(5, 95), 1, true),
            ],
            vec![loop_(square(20, 80), 2, true)],
        ],
        holes: vec![
            vec![loop_(square(30, 31), 0, false)],
            Vec::new(),
            Vec::new(),
        ],
    };
    let nested = nest(buckets);
    let first_depth_one = &nested.roots[0].children[0];
    assert_eq!(first_depth_one.polygon.points()[0], Point::new(10, 10));
    assert_eq!(
        first_depth_one.children[0].polygon.points()[0],
        Point::new(20, 20)
    );
    assert_eq!(
        first_depth_one.children[0].children[0].polygon.points()[0],
        Point::new(30, 30)
    );
    assert_eq!(
        nested.roots[0].children[1].polygon.points()[0],
        Point::new(5, 5)
    );
}

#[test]
fn task22o4_empty_buckets_preserve_no_roots_or_diagnostics() {
    let nested = nest(LoopBuckets {
        contours: Vec::new(),
        holes: Vec::new(),
    });
    assert!(nested.roots.is_empty());
    assert!(nested.contours.is_empty());
    assert!(nested.holes.is_empty());
}
