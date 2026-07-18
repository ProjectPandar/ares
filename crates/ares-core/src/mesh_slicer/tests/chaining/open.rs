use super::{edge, line, points, vertex};
use crate::mesh_slicer::chain_lines_by_triangle_connectivity;

#[test]
fn task22c_open_chain_preserves_endpoints_points_length_and_state() {
    let layer = chain_lines_by_triangle_connectivity(vec![
        line((0, 0, vertex(10)), (3, 4, edge(20))),
        line((3, 4, edge(20)), (6, 8, vertex(30))),
    ]);

    assert!(layer.polygons().is_empty());
    assert_eq!(layer.open_polylines().len(), 1);
    let polyline = &layer.open_polylines()[0];
    assert_eq!(polyline.start(), vertex(10));
    assert_eq!(polyline.end(), vertex(30));
    assert_eq!(polyline.points(), points(&[(0, 0), (3, 4), (6, 8)]));
    assert_eq!(polyline.length(), 10.0);
    assert!(!polyline.consumed());
}

#[test]
fn task22c_open_length_avoids_extreme_coordinate_overflow() {
    let layer = chain_lines_by_triangle_connectivity(vec![line(
        (i64::MIN, i64::MIN, vertex(1)),
        (i64::MAX, i64::MAX, vertex(2)),
    )]);
    let delta = (i128::from(i64::MAX) - i128::from(i64::MIN)) as f64;
    let expected = (delta * delta + delta * delta).sqrt();

    assert_eq!(layer.open_polylines().len(), 1);
    assert!(layer.open_polylines()[0].length().is_finite());
    assert_eq!(layer.open_polylines()[0].length(), expected);
}
