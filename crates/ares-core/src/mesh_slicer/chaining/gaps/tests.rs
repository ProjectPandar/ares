use crate::{
    geometry::{Point, Polygon},
    mesh_slicer::EndpointReference,
};

use super::spatial::{EndpointSide, RadiusGrid};
use super::{chain_open_polylines_close_gaps, endpoint_key, restore_open_seed, sorted_gap_indices};
use crate::mesh_slicer::chaining::OpenPolyline;

fn vertex(id: u32) -> EndpointReference {
    EndpointReference::Vertex(id)
}

fn edge(id: u32) -> EndpointReference {
    EndpointReference::Edge(id)
}

fn points(values: &[(i64, i64)]) -> Vec<Point> {
    values.iter().map(|&(x, y)| Point::new(x, y)).collect()
}

fn open(start: EndpointReference, end: EndpointReference, values: &[(i64, i64)]) -> OpenPolyline {
    OpenPolyline::new(start, end, points(values))
}

fn run(
    mut polylines: Vec<OpenPolyline>,
    radius: i64,
    reversed: bool,
) -> (Vec<OpenPolyline>, Vec<Polygon>) {
    let mut polygons = Vec::new();
    chain_open_polylines_close_gaps(&mut polylines, &mut polygons, radius, reversed);
    (polylines, polygons)
}

#[test]
fn task22d_gap_recomputes_lengths_then_uses_original_index_ties() {
    let mut polylines = vec![
        open(vertex(1), edge(1), &[(0, 0), (100, 0)]),
        open(vertex(2), edge(2), &[(0, 0), (5, 0)]),
        open(vertex(3), edge(3), &[(0, 0), (3, 4)]),
        open(vertex(4), edge(4), &[(0, 0), (200, 0)]),
    ];
    polylines[0].length = 1.0;
    polylines[1].length = 99.0;
    polylines[2].length = 0.0;
    polylines[3].consumed = true;

    assert_eq!(sorted_gap_indices(&mut polylines), [0, 1, 2]);
    assert_eq!(polylines[0].length, 100.0);
    assert_eq!(polylines[1].length, 5.0);
    assert_eq!(polylines[2].length, 5.0);
    assert_eq!(polylines[3].length, 200.0);
}

#[test]
fn task22d_gap_strict_closure_radius_is_exact_for_both_scales() {
    for radius in [2_000_000_i64, 199_999] {
        let start = (i64::MAX - radius, i64::MIN + radius);
        let middle = (i64::MAX - radius, i64::MIN + 2 * radius);

        let equality = open(vertex(1), vertex(2), &[start, middle, (i64::MAX, start.1)]);
        let (equality, polygons) = run(vec![equality], radius, false);
        assert!(polygons.is_empty());
        assert!(!equality[0].consumed);

        let inside = open(
            vertex(1),
            vertex(2),
            &[start, middle, (i64::MAX - 1, start.1)],
        );
        let (_, polygons) = run(vec![inside], radius, false);
        assert_eq!(polygons.len(), 1);
        assert_eq!(polygons[0].points().len(), 3);
    }
}

#[test]
fn task22d_gap_closure_heuristic_preserves_source_branch_order() {
    let no_candidate = open(vertex(1), vertex(2), &[(0, 0), (13, 0), (6, 0)]);
    let (_, polygons) = run(vec![no_candidate], 10, false);
    assert_eq!(polygons[0].points(), points(&[(0, 0), (13, 0), (6, 0)]));

    let equality = vec![
        open(vertex(1), vertex(2), &[(0, 0), (13, 0), (6, 0)]),
        open(vertex(3), vertex(4), &[(13, 0), (23, 0)]),
    ];
    let (equality, polygons) = run(equality, 10, false);
    assert!(polygons.is_empty());
    assert_eq!(
        equality[0].points,
        points(&[(0, 0), (13, 0), (6, 0), (13, 0), (23, 0)])
    );
    assert_eq!(equality[0].length, 20.0);
    assert_eq!(equality[1].length, 10.0);
    assert!(equality[1].consumed);

    let above_threshold = vec![
        open(vertex(1), vertex(2), &[(0, 0), (12, 0), (7, 0)]),
        open(vertex(3), vertex(4), &[(15, 0), (25, 0)]),
    ];
    let (above_threshold, polygons) = run(above_threshold, 10, false);
    assert!(polygons.is_empty());
    assert_eq!(
        above_threshold[0].points,
        points(&[(0, 0), (12, 0), (7, 0), (15, 0), (25, 0)])
    );
    assert_eq!(above_threshold[0].length, 17.0);
    assert!(above_threshold[1].consumed);

    let passes = vec![
        open(vertex(1), vertex(2), &[(0, 0), (14, 0), (6, 0)]),
        open(vertex(3), vertex(4), &[(13, 0), (23, 0)]),
    ];
    let (passes, polygons) = run(passes, 10, false);
    assert_eq!(polygons.len(), 1);
    assert_eq!(polygons[0].points(), points(&[(0, 0), (14, 0), (6, 0)]));
    assert!(!passes[1].consumed);

    for candidate_x in [12, 11] {
        let branches = vec![
            open(vertex(1), vertex(2), &[(0, 0), (13, 0), (6, 0)]),
            open(
                vertex(3),
                vertex(4),
                &[(candidate_x, 0), (candidate_x + 10, 0)],
            ),
        ];
        let (branches, polygons) = run(branches, 10, false);
        assert_eq!(polygons.len(), 1);
        assert_eq!(polygons[0].points(), points(&[(0, 0), (13, 0), (6, 0)]));
        assert!(!branches[1].consumed);
    }
}

#[test]
fn task22d_gap_attachment_retains_only_nonzero_junctions_and_stale_metadata() {
    let equal = vec![
        open(vertex(1), edge(2), &[(0, 0), (10, 0)]),
        open(edge(2), vertex(3), &[(10, 0), (15, 0)]),
    ];
    let (equal, polygons) = run(equal, 5, false);
    assert!(polygons.is_empty());
    assert_eq!(equal[0].points, points(&[(0, 0), (10, 0), (15, 0)]));

    let nonzero = vec![
        open(vertex(1), edge(2), &[(0, 0), (10, 0)]),
        open(vertex(3), vertex(4), &[(15, 0), (11, 0)]),
    ];
    let (nonzero, polygons) = run(nonzero, 5, true);
    assert!(polygons.is_empty());
    assert_eq!(
        nonzero[0].points,
        points(&[(0, 0), (10, 0), (11, 0), (15, 0)])
    );
    assert_eq!(nonzero[0].end, edge(2));
    assert_eq!(nonzero[0].length, 10.0);
    assert!(!nonzero[0].consumed);
    assert!(nonzero[1].points.is_empty());
    assert_eq!(nonzero[1].length, 4.0);
    assert!(nonzero[1].consumed);
}

#[test]
fn task22d_gap_restore_reinserts_the_changed_reversed_end() {
    let mut polylines = vec![open(vertex(1), edge(2), &[(0, 0), (15, 0)])];
    polylines[0].consumed = true;
    let mut grid = RadiusGrid::new(5);
    let end = endpoint_key(0, EndpointSide::End);
    grid.insert(end, Point::new(10, 0));
    assert!(grid.remove(end));

    restore_open_seed(&mut polylines, &mut grid, 0, true);

    assert!(!polylines[0].consumed);
    assert_eq!(grid.find(Point::new(15, 0), |_| true).unwrap().key, end);
    assert!(grid.find(Point::new(10, 0), |_| true).is_none());
}

#[test]
fn task22d_gap_reversed_area_gate_requires_multiple_joined_segments() {
    let joined = vec![
        open(vertex(1), edge(2), &[(0, 0), (0, 10), (10, 10)]),
        open(vertex(3), vertex(4), &[(0, 1), (10, 0), (10, 9)]),
    ];
    let (_, polygons) = run(joined, 2, true);
    assert_eq!(
        polygons[0].points(),
        points(&[(0, 1), (10, 0), (10, 9), (10, 10), (0, 10), (0, 0)])
    );

    let single = open(vertex(5), vertex(6), &[(0, 0), (0, 10), (10, 0), (0, 1)]);
    let (_, polygons) = run(vec![single], 2, true);
    assert_eq!(
        polygons[0].points(),
        points(&[(0, 0), (0, 10), (10, 0), (0, 1)])
    );
}

#[test]
fn task22d_gap_zero_distance_closure_pops_then_drops_short_results() {
    let short = open(vertex(1), vertex(2), &[(0, 0), (4, 0), (0, 0)]);
    let (_, polygons) = run(vec![short], 5, false);
    assert!(polygons.is_empty());

    let retained = open(vertex(1), vertex(2), &[(0, 0), (4, 0), (0, 4), (0, 0)]);
    let (_, polygons) = run(vec![retained], 5, false);
    assert_eq!(polygons[0].points(), points(&[(0, 0), (4, 0), (0, 4)]));
}
