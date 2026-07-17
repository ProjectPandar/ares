use crate::{
    geometry::Point,
    mesh_slicer::{
        ChainedLayer, EndpointReference, FacetEdgeType, IntersectionLine, IntersectionPoint,
        chain_lines_by_triangle_connectivity,
    },
};

mod identity;
mod open;

fn line(a: (i64, i64, EndpointReference), b: (i64, i64, EndpointReference)) -> IntersectionLine {
    IntersectionLine::new(
        IntersectionPoint::new(Point::new(a.0, a.1), a.2),
        IntersectionPoint::new(Point::new(b.0, b.1), b.2),
        FacetEdgeType::General,
    )
}

fn edge(id: u32) -> EndpointReference {
    EndpointReference::Edge(id)
}

fn vertex(id: u32) -> EndpointReference {
    EndpointReference::Vertex(id)
}

fn points(values: &[(i64, i64)]) -> Vec<Point> {
    values.iter().map(|&(x, y)| Point::new(x, y)).collect()
}

fn assert_conservation(layer: &ChainedLayer, input_lines: usize) {
    let closed_edges = layer
        .polygons()
        .iter()
        .map(|polygon| polygon.points().len())
        .sum::<usize>();
    let open_edges = layer
        .open_polylines()
        .iter()
        .map(|polyline| polyline.points().len() - 1)
        .sum::<usize>();
    assert_eq!(closed_edges + open_edges, input_lines);
}

#[test]
fn task22c_edge_cycle_forms_one_ordered_polygon() {
    let layer = chain_lines_by_triangle_connectivity(vec![
        line((0, 0, edge(0)), (4, 0, edge(1))),
        line((4, 0, edge(1)), (1, 3, edge(2))),
        line((1, 3, edge(2)), (0, 0, edge(0))),
    ]);

    assert_eq!(layer.polygons().len(), 1);
    assert_eq!(
        layer.polygons()[0].points(),
        points(&[(0, 0), (4, 0), (1, 3)])
    );
    assert!(layer.open_polylines().is_empty());
}

#[test]
fn task22c_vertex_cycle_forms_one_ordered_polygon() {
    let layer = chain_lines_by_triangle_connectivity(vec![
        line((2, 2, vertex(10)), (7, 2, vertex(11))),
        line((7, 2, vertex(11)), (2, 8, vertex(12))),
        line((2, 8, vertex(12)), (2, 2, vertex(10))),
    ]);

    assert_eq!(layer.polygons().len(), 1);
    assert_eq!(
        layer.polygons()[0].points(),
        points(&[(2, 2), (7, 2), (2, 8)])
    );
    assert!(layer.open_polylines().is_empty());
}

#[test]
fn task22c_components_keep_seed_order() {
    let layer = chain_lines_by_triangle_connectivity(vec![
        line((0, 0, edge(1)), (2, 0, edge(2))),
        line((9, 9, vertex(20)), (9, 9, vertex(20))),
        line((2, 0, edge(2)), (0, 0, edge(1))),
    ]);

    assert_eq!(layer.polygons().len(), 2);
    assert_eq!(layer.polygons()[0].points(), points(&[(0, 0), (2, 0)]));
    assert_eq!(layer.polygons()[1].points(), points(&[(9, 9)]));
    assert!(layer.open_polylines().is_empty());
}

#[test]
fn task22c_equal_start_candidates_use_input_fifo() {
    let layer = chain_lines_by_triangle_connectivity(vec![
        line((0, 0, edge(1)), (1, 0, edge(2))),
        line((1, 0, edge(2)), (2, 0, edge(3))),
        line((1, 0, edge(2)), (1, 2, edge(4))),
    ]);

    assert!(layer.polygons().is_empty());
    assert_eq!(layer.open_polylines().len(), 2);
    assert_eq!(
        layer.open_polylines()[0].points(),
        points(&[(0, 0), (1, 0), (2, 0)])
    );
    assert_eq!(layer.open_polylines()[0].end(), edge(3));
    assert_eq!(
        layer.open_polylines()[1].points(),
        points(&[(1, 0), (1, 2)])
    );
}

#[test]
fn task22c_empty_single_open_and_single_closed_layers_are_retained() {
    let empty = chain_lines_by_triangle_connectivity(Vec::new());
    assert!(empty.polygons().is_empty());
    assert!(empty.open_polylines().is_empty());

    let open = chain_lines_by_triangle_connectivity(vec![line((1, 2, vertex(1)), (3, 4, edge(2)))]);
    assert!(open.polygons().is_empty());
    assert_eq!(open.open_polylines().len(), 1);
    assert_eq!(open.open_polylines()[0].points(), points(&[(1, 2), (3, 4)]));

    let closed = chain_lines_by_triangle_connectivity(vec![line((4, 5, edge(9)), (4, 5, edge(9)))]);
    assert_eq!(closed.polygons().len(), 1);
    assert_eq!(closed.polygons()[0].points(), points(&[(4, 5)]));
    assert!(closed.open_polylines().is_empty());
}

#[test]
fn task22c_mixed_components_conserve_every_input_edge() {
    let lines = vec![
        line((0, 0, edge(1)), (2, 0, edge(2))),
        line((10, 0, vertex(10)), (13, 4, vertex(11))),
        line((2, 0, edge(2)), (0, 0, edge(1))),
        line((13, 4, vertex(11)), (16, 8, edge(12))),
        line((20, 20, vertex(20)), (20, 20, vertex(20))),
    ];
    let input_lines = lines.len();

    let layer = chain_lines_by_triangle_connectivity(lines);

    assert_eq!(layer.polygons().len(), 2);
    assert_eq!(layer.open_polylines().len(), 1);
    assert_conservation(&layer, input_lines);
}
