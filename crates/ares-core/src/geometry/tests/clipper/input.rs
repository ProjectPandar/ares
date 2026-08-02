use super::helpers::{point, polygon, square};
use crate::geometry::Polygon;
use crate::geometry::clipper::{Clipper, ClipperOptions, PathRole};

#[test]
fn task22f_closed_input_ignores_empty_one_two_point_and_flat_paths() {
    let paths = vec![
        Polygon::new(Vec::new()),
        polygon(&[(0, 0)]),
        polygon(&[(0, 0), (10, 0)]),
        polygon(&[(0, 0), (5, 0), (10, 0)]),
    ];
    let mut clipper = Clipper::new(ClipperOptions::default());

    assert_eq!(
        clipper.add_closed_paths(&paths, PathRole::Subject),
        Ok(false)
    );
    let snapshot = clipper.input_snapshot();
    assert!(snapshot.edges.is_empty());
    assert!(snapshot.minima.is_empty());
}

#[test]
fn task22f_closed_input_keeps_valid_siblings_among_degenerate_paths() {
    let paths = vec![
        Polygon::new(Vec::new()),
        square(),
        polygon(&[(20, 0), (25, 0), (30, 0)]),
        polygon(&[(40, 0), (50, 0), (45, 10)]),
    ];
    let mut clipper = Clipper::new(ClipperOptions::default());

    assert_eq!(
        clipper.add_closed_paths(&paths, PathRole::Subject),
        Ok(true)
    );
    let snapshot = clipper.input_snapshot();
    assert_eq!(snapshot.edges.len(), 7);
    assert_eq!(snapshot.minima.len(), 2);
}

#[test]
fn task22f_closed_input_trims_repeated_terminal_points_before_allocation() {
    for path in [
        polygon(&[(0, 0), (10, 0), (10, 10), (0, 10), (0, 0), (0, 0)]),
        polygon(&[(0, 0), (10, 0), (10, 10), (0, 10), (0, 10), (0, 10)]),
    ] {
        let mut clipper = Clipper::new(ClipperOptions::default());
        assert_eq!(clipper.add_closed_path(&path, PathRole::Subject), Ok(true));
        let snapshot = clipper.input_snapshot();
        assert_eq!(snapshot.edges.len(), 4);
        assert!(snapshot.edges.iter().all(|edge| !edge.removed));
    }
}

#[test]
fn task22f_closed_input_removes_adjacent_duplicate_without_renumbering_edge_ids() {
    let duplicate = polygon(&[(0, 0), (10, 0), (10, 0), (10, 10), (0, 10)]);
    let mut clipper = Clipper::new(ClipperOptions::default());

    assert_eq!(
        clipper.add_closed_path(&duplicate, PathRole::Subject),
        Ok(true)
    );
    assert_eq!(
        clipper.add_closed_path(&square(), PathRole::Subject),
        Ok(true)
    );

    let edges = &clipper.input_snapshot().edges;
    assert_eq!(edges.len(), 9);
    assert!(edges[1].removed);
    assert_eq!((edges[0].previous, edges[0].next), (Some(4), Some(2)));
    assert_eq!((edges[2].previous, edges[2].next), (Some(0), Some(3)));
    assert_eq!((edges[3].previous, edges[3].next), (Some(2), Some(4)));
    assert_eq!((edges[4].previous, edges[4].next), (Some(3), Some(0)));
    assert_eq!(edges[5].current, Some(point(0, 0)));
}

#[test]
fn task22f_closed_input_preserve_collinear_retains_only_between_vertices() {
    let between = polygon(&[(0, 0), (5, 0), (10, 0), (10, 10), (0, 10)]);
    let spike = polygon(&[(0, 0), (10, 0), (5, 0), (10, 10), (0, 10)]);

    let mut default_clipper = Clipper::new(ClipperOptions::default());
    assert_eq!(
        default_clipper.add_closed_path(&between, PathRole::Subject),
        Ok(true)
    );
    assert!(default_clipper.input_snapshot().edges[1].removed);

    let mut preserving = Clipper::new(ClipperOptions {
        preserve_collinear: true,
        ..ClipperOptions::default()
    });
    assert_eq!(
        preserving.add_closed_path(&between, PathRole::Subject),
        Ok(true)
    );
    assert!(
        preserving
            .input_snapshot()
            .edges
            .iter()
            .all(|edge| !edge.removed)
    );

    let mut preserving_spike = Clipper::new(ClipperOptions {
        preserve_collinear: true,
        ..ClipperOptions::default()
    });
    assert_eq!(
        preserving_spike.add_closed_path(&spike, PathRole::Subject),
        Ok(true)
    );
    assert!(preserving_spike.input_snapshot().edges[1].removed);
}

#[test]
fn task22f_closed_input_builds_minima_and_lml_in_source_order() {
    let diamond = polygon(&[(0, 0), (10, 10), (0, 20), (-10, 10)]);
    let shifted = polygon(&[(30, 0), (40, 10), (30, 20), (20, 10)]);
    let mut clipper = Clipper::new(ClipperOptions::default());

    assert_eq!(
        clipper.add_closed_paths(&[diamond, shifted], PathRole::Subject),
        Ok(true)
    );
    let snapshot = clipper.input_snapshot();
    assert_eq!(snapshot.minima.len(), 2);
    assert_eq!(
        (
            snapshot.minima[0].y,
            snapshot.minima[0].left,
            snapshot.minima[0].right,
        ),
        (20, Some(2), Some(1))
    );
    assert_eq!(snapshot.edges[2].next_in_lml, Some(3));
    assert_eq!(snapshot.edges[1].next_in_lml, Some(0));
    assert_eq!(
        snapshot
            .minima
            .iter()
            .map(|minimum| (minimum.y, minimum.left, minimum.right))
            .collect::<Vec<_>>(),
        vec![(20, Some(2), Some(1)), (20, Some(6), Some(5))]
    );
}

#[test]
fn task22f_closed_input_normalizes_horizontal_rectangle_and_equal_height_order() {
    let rectangle = square();
    let mut clipper = Clipper::new(ClipperOptions::default());

    assert_eq!(
        clipper.add_closed_path(&rectangle, PathRole::Subject),
        Ok(true)
    );
    let snapshot = clipper.input_snapshot();
    assert_eq!(snapshot.edges.len(), 4);
    assert_eq!(
        (
            snapshot.edges[0].bottom,
            snapshot.edges[0].top,
            snapshot.edges[0].dx,
            snapshot.edges[0].wind_delta,
            snapshot.edges[0].next_in_lml,
        ),
        (
            Some(point(0, 0)),
            Some(point(10, 0)),
            Some(-1.0e40),
            Some(0),
            None,
        )
    );
    assert_eq!(
        (
            snapshot.edges[1].bottom,
            snapshot.edges[1].top,
            snapshot.edges[1].dx,
            snapshot.edges[1].wind_delta,
            snapshot.edges[1].next_in_lml,
        ),
        (
            Some(point(10, 10)),
            Some(point(10, 0)),
            Some(0.0),
            Some(0),
            None,
        )
    );
    assert_eq!(
        (
            snapshot.edges[2].bottom,
            snapshot.edges[2].top,
            snapshot.edges[2].dx,
            snapshot.edges[2].wind_delta,
            snapshot.edges[2].next_in_lml,
        ),
        (
            Some(point(0, 10)),
            Some(point(10, 10)),
            Some(-1.0e40),
            Some(-1),
            Some(1),
        )
    );
    assert_eq!(
        (
            snapshot.edges[3].bottom,
            snapshot.edges[3].top,
            snapshot.edges[3].dx,
            snapshot.edges[3].wind_delta,
            snapshot.edges[3].next_in_lml,
        ),
        (
            Some(point(0, 10)),
            Some(point(0, 0)),
            Some(0.0),
            Some(1),
            Some(0),
        )
    );
    assert_eq!(
        (
            snapshot.minima[0].y,
            snapshot.minima[0].left,
            snapshot.minima[0].right,
        ),
        (10, Some(3), Some(2))
    );
}
