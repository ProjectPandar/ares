use crate::{
    geometry::{Point, Polygon},
    mesh_slicer::{
        EndpointReference, FacetEdgeType, IntersectionLine, IntersectionPoint,
        chain_lines_by_triangle_connectivity,
    },
};

use super::{
    EndpointRecord, EndpointSide, ExactIndex, chain_open_polylines_exact, reference_key,
    sorted_exact_indices,
};
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

fn open(
    start: EndpointReference,
    end: EndpointReference,
    values: &[(i64, i64)],
    length: f64,
) -> OpenPolyline {
    let mut polyline = OpenPolyline::new(start, end, points(values));
    polyline.length = length;
    polyline
}

fn line(a: (i64, i64, EndpointReference), b: (i64, i64, EndpointReference)) -> IntersectionLine {
    IntersectionLine::new(
        IntersectionPoint::new(Point::new(a.0, a.1), a.2),
        IntersectionPoint::new(Point::new(b.0, b.1), b.2),
        FacetEdgeType::General,
    )
}

#[test]
fn task22d_exact_reference_key_preserves_zero_collision_and_extremes() {
    assert_eq!(reference_key(vertex(0)), 0);
    assert_eq!(reference_key(edge(0)), 0);
    assert_eq!(reference_key(vertex(u32::MAX)), i64::from(u32::MAX));
    assert_eq!(reference_key(edge(u32::MAX)), -i64::from(u32::MAX));
}

#[test]
fn task22d_exact_sort_uses_cached_length_then_original_index() {
    let mut polylines = vec![
        open(vertex(1), edge(1), &[(0, 0), (100, 0)], 1.0),
        open(vertex(2), edge(2), &[(0, 0), (1, 0)], 5.0),
        open(vertex(3), edge(3), &[(0, 0), (2, 0)], 5.0),
        open(vertex(4), edge(4), &[(0, 0), (3, 0)], 99.0),
    ];
    polylines[3].consumed = true;

    assert_eq!(sorted_exact_indices(&polylines), [1, 2, 0]);
}

#[test]
fn task22d_exact_false_reuses_stale_end_and_destroys_all_matches() {
    let mut polylines = vec![
        open(vertex(10), edge(7), &[(0, 0), (1, 0)], 100.0),
        open(edge(7), vertex(20), &[(999, 999), (2, 0)], 20.0),
        open(edge(7), vertex(30), &[(-999, -999), (3, 0)], 10.0),
    ];
    let mut polygons = Vec::new();

    chain_open_polylines_exact(&mut polylines, &mut polygons, false);

    assert!(polygons.is_empty());
    assert_eq!(
        polylines[0].points,
        points(&[(0, 0), (1, 0), (2, 0), (3, 0)])
    );
    assert_eq!(polylines[0].end, edge(7));
    assert_eq!(polylines[0].length, 130.0);
    assert!(!polylines[0].consumed);
    for polyline in &polylines[1..] {
        assert!(polyline.points.is_empty());
        assert_eq!(polyline.length, 0.0);
        assert!(polyline.consumed);
    }
}

#[test]
fn task22d_exact_true_attaches_start_then_end_and_closes() {
    let mut polylines = vec![
        open(vertex(1), edge(2), &[(0, 0), (2, 0)], 100.0),
        open(edge(2), vertex(3), &[(777, 777), (2, 2)], 20.0),
        open(vertex(1), vertex(3), &[(0, 0), (888, 888)], 10.0),
    ];
    let sentinel = Polygon::new(points(&[(9, 9), (10, 9), (9, 10)]));
    let mut polygons = vec![sentinel.clone()];

    chain_open_polylines_exact(&mut polylines, &mut polygons, true);

    assert_eq!(polygons[0], sentinel);
    assert_eq!(polygons[1].points(), points(&[(0, 0), (2, 0), (2, 2)]));
    assert_eq!(polylines[0].length, 130.0);
    assert!(polylines.iter().all(|polyline| polyline.consumed));
    assert!(polylines.iter().all(|polyline| polyline.points.is_empty()));
    assert_eq!(polylines[1].length, 0.0);
    assert_eq!(polylines[2].length, 0.0);
}

#[test]
fn task22d_exact_candidate_tie_is_index_then_start_before_end() {
    let mut polylines = vec![
        open(vertex(9), edge(5), &[(0, 0), (1, 0)], 100.0),
        open(edge(5), edge(5), &[(10, 10), (20, 0), (30, 0)], 10.0),
        open(edge(5), vertex(9), &[(40, 40), (0, 0)], 5.0),
    ];
    let mut polygons = Vec::new();

    chain_open_polylines_exact(&mut polylines, &mut polygons, true);

    assert_eq!(polygons.len(), 1);
    assert_eq!(
        polygons[0].points(),
        points(&[(0, 0), (1, 0), (20, 0), (30, 0)])
    );
}

#[test]
fn task22d_exact_zero_key_collision_joins_without_cross_tag_closure() {
    let mut polylines = vec![
        open(vertex(0), vertex(5), &[(0, 0), (1, 0)], 100.0),
        open(edge(0), vertex(5), &[(2, 0), (1, 0)], 20.0),
        open(vertex(0), vertex(7), &[(999, 999), (3, 0)], 10.0),
    ];
    let mut polygons = Vec::new();

    chain_open_polylines_exact(&mut polylines, &mut polygons, true);

    assert!(polygons.is_empty());
    assert_eq!(polylines[0].start, vertex(0));
    assert_eq!(polylines[0].end, vertex(7));
    assert_eq!(
        polylines[0].points,
        points(&[(0, 0), (1, 0), (2, 0), (3, 0)])
    );
    assert!(!polylines[0].consumed);
}

#[test]
fn task22d_exact_index_moves_live_end_record() {
    let polylines = vec![open(vertex(1), edge(2), &[(0, 0), (2, 0)], 100.0)];
    let mut index = ExactIndex::new(&polylines, true);

    index.move_end(0, edge(2), vertex(3));

    assert_eq!(index.first_other(edge(2), usize::MAX), None);
    assert_eq!(
        index.first_other(vertex(3), usize::MAX),
        Some(EndpointRecord {
            polyline_index: 0,
            side: EndpointSide::End,
        })
    );
}

#[test]
fn task22d_exact_closure_pop_drop_and_area_gates_match_source() {
    let mut false_polylines = vec![
        open(edge(9), edge(9), &[(0, 0), (0, 3)], 100.0),
        open(edge(9), vertex(2), &[(123, 123), (4, 0), (999, 999)], 10.0),
    ];
    let mut polygons = Vec::new();
    chain_open_polylines_exact(&mut false_polylines, &mut polygons, false);
    assert_eq!(polygons[0].points(), points(&[(0, 0), (0, 3), (4, 0)]));

    let m = u32::MAX;
    let p0 = (i64::MIN, i64::MIN);
    let p1 = (i64::MIN, i64::MAX);
    let p2 = (i64::MAX, i64::MIN);
    let mut true_polylines = vec![
        open(vertex(m), edge(m), &[p0, p1], 100.0),
        open(edge(m), vertex(m), &[p1, p2, p0], 10.0),
    ];
    chain_open_polylines_exact(&mut true_polylines, &mut polygons, true);
    assert_eq!(polygons[1].points(), points(&[p2, p1, p0]));

    let mut short = vec![
        open(edge(1), edge(1), &[(0, 0)], 100.0),
        open(edge(1), vertex(2), &[(9, 9), (1, 1)], 10.0),
    ];
    chain_open_polylines_exact(&mut short, &mut polygons, false);
    assert_eq!(polygons.len(), 2);
    assert!(short[0].points.is_empty());
    assert!(short[0].consumed);
}

#[test]
fn task22d_exact_accepts_task22c_open_state() {
    let chained = chain_lines_by_triangle_connectivity(vec![
        line((0, 0, vertex(1)), (2, 0, edge(2))),
        line((2, 2, vertex(3)), (2, 0, edge(2))),
        line((2, 2, vertex(3)), (0, 0, vertex(1))),
    ]);
    let (mut polygons, mut polylines) = chained.into_parts();
    assert_eq!(polylines.len(), 3);

    chain_open_polylines_exact(&mut polylines, &mut polygons, true);

    assert_eq!(polygons.len(), 1);
    assert_eq!(polygons[0].points(), points(&[(2, 2), (0, 0), (2, 0)]));
    assert!(polylines.iter().all(|polyline| polyline.consumed));
    assert!(polylines.iter().all(|polyline| polyline.points.is_empty()));
}
