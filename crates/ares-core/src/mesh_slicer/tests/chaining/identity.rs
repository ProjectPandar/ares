use super::{edge, line, points, vertex};
use crate::mesh_slicer::chain_lines_by_triangle_connectivity;

#[test]
fn task22c_equal_coordinates_with_different_references_do_not_connect() {
    let layer = chain_lines_by_triangle_connectivity(vec![
        line((0, 0, vertex(1)), (1, 0, edge(9))),
        line((1, 0, edge(10)), (2, 0, vertex(2))),
    ]);

    assert!(layer.polygons().is_empty());
    assert_eq!(layer.open_polylines().len(), 2);
    assert_eq!(
        layer.open_polylines()[0].points(),
        points(&[(0, 0), (1, 0)])
    );
    assert_eq!(
        layer.open_polylines()[1].points(),
        points(&[(1, 0), (2, 0)])
    );
}

#[test]
fn task22c_vertex_and_edge_with_same_numeric_id_do_not_connect() {
    let layer = chain_lines_by_triangle_connectivity(vec![
        line((0, 0, vertex(1)), (1, 0, vertex(7))),
        line((1, 0, edge(7)), (2, 0, vertex(2))),
    ]);

    assert!(layer.polygons().is_empty());
    assert_eq!(layer.open_polylines().len(), 2);
    assert_eq!(layer.open_polylines()[0].end(), vertex(7));
    assert_eq!(layer.open_polylines()[1].start(), edge(7));
}

#[test]
fn task22c_successor_is_never_reversed() {
    let layer = chain_lines_by_triangle_connectivity(vec![
        line((0, 0, vertex(1)), (1, 0, edge(2))),
        line((2, 0, vertex(3)), (1, 0, edge(2))),
    ]);

    assert!(layer.polygons().is_empty());
    assert_eq!(layer.open_polylines().len(), 2);
    assert_eq!(
        layer.open_polylines()[0].points(),
        points(&[(0, 0), (1, 0)])
    );
    assert_eq!(
        layer.open_polylines()[1].points(),
        points(&[(2, 0), (1, 0)])
    );
}

#[cfg(debug_assertions)]
#[test]
fn task22c_matching_identities_require_matching_coordinates_in_debug_builds() {
    let successor_mismatch = std::panic::catch_unwind(|| {
        chain_lines_by_triangle_connectivity(vec![
            line((0, 0, vertex(1)), (1, 0, edge(7))),
            line((9, 9, edge(7)), (2, 0, vertex(2))),
        ])
    });
    assert!(successor_mismatch.is_err());

    let closure_mismatch = std::panic::catch_unwind(|| {
        chain_lines_by_triangle_connectivity(vec![line((0, 0, edge(7)), (1, 0, edge(7)))])
    });
    assert!(closure_mismatch.is_err());
}
