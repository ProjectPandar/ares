use crate::geometry::Coord;

use super::super::{
    EndpointReference, FacetEdgeType, IntersectionLine, IntersectionPoint, intersect_facet,
    intersection::lowest_vertex_index,
};

type EndpointSignature = (Coord, Coord, EndpointReference);
type LineSignature = (EndpointSignature, EndpointSignature, FacetEdgeType);

fn endpoint_signature(endpoint: IntersectionPoint) -> EndpointSignature {
    let point = endpoint.point();
    (point.x(), point.y(), endpoint.reference())
}

fn line_signature(line: IntersectionLine) -> LineSignature {
    (
        endpoint_signature(line.a()),
        endpoint_signature(line.b()),
        line.edge_type(),
    )
}

#[test]
fn task22b_facet_crossing_preserves_direction_and_endpoint_provenance() {
    let vertices = [[0.0, 0.0, -1.0], [0.0, 10.0, 1.0], [20.0, 0.0, 1.0]];
    let line = intersect_facet(0.0, &vertices, [20, 5, 11], [100, 101, 102]).unwrap();

    assert_eq!(
        line_signature(line),
        (
            (10, 0, EndpointReference::Edge(102)),
            (0, 5, EndpointReference::Edge(100)),
            FacetEdgeType::General,
        )
    );
}

#[test]
fn task22b_facet_conversion_distinguishes_vertex_truncation_from_interior_floor_plus_half() {
    let vertices = [[1.9, -1.9, 0.0], [2.0, -3.0, -1.0], [3.0, -2.0, 1.0]];
    let line = intersect_facet(0.0, &vertices, [30, 10, 20], [500, 501, 502]).unwrap();

    assert_eq!(
        line_signature(line),
        (
            (1, -1, EndpointReference::Vertex(30)),
            (3, -2, EndpointReference::Edge(501)),
            FacetEdgeType::General,
        )
    );

    let precision_vertices = [
        [
            f32::from_bits(0xcdd9_f69d),
            0.0,
            f32::from_bits(0xc710_e603),
        ],
        [
            f32::from_bits(0x4d70_81e4),
            0.0,
            f32::from_bits(0x4870_ecd3),
        ],
        [123.0, 456.0, f32::from_bits(0xc709_ef0e)],
    ];
    let precision_line = intersect_facet(
        f32::from_bits(0xc709_ef0e),
        &precision_vertices,
        [20, 5, 11],
        [910, 911, 912],
    )
    .unwrap();
    assert_eq!(
        line_signature(precision_line),
        (
            (123, 456, EndpointReference::Vertex(11)),
            (-452_646_172, 0, EndpointReference::Edge(910)),
            FacetEdgeType::General,
        )
    );
}

#[test]
fn task22b_facet_single_vertex_dedup_uses_exact_id_and_strict_plane_equality() {
    assert_eq!(
        lowest_vertex_index(&[[0.0, 0.0, 2.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0]], 1.0),
        1
    );
    assert_eq!(
        lowest_vertex_index(&[[0.0, 0.0, 2.0], [0.0, 0.0, 3.0], [0.0, 0.0, 1.0]], 1.0),
        2
    );
    assert_eq!(
        lowest_vertex_index(&[[0.0, 0.0, 1.0], [0.0, 0.0, 2.0], [0.0, 0.0, 1.0]], 1.0),
        2
    );
    assert_eq!(
        lowest_vertex_index(&[[0.0, 0.0, 1.0], [0.0, 0.0, 2.0], [0.0, 0.0, 3.0]], 1.0),
        0
    );

    let near_plane = f32::from_bits(1.0_f32.to_bits() + 1);
    let vertices = [[3.0, 4.0, 1.0], [0.0, 0.0, 0.0], [10.0, 0.0, near_plane]];
    let line = intersect_facet(1.0, &vertices, [30, 10, 20], [600, 601, 602]).unwrap();

    assert_eq!(
        line_signature(line),
        (
            (3, 4, EndpointReference::Vertex(30)),
            (10, 0, EndpointReference::Edge(601)),
            FacetEdgeType::General,
        )
    );
}

#[test]
fn task22b_facet_top_bottom_and_horizontal_ownership_matches_orca() {
    let top = [[1.9, -1.9, 1.0], [4.9, 2.9, 1.0], [0.0, 0.0, 0.0]];
    let top_line = intersect_facet(1.0, &top, [40, 41, 42], [700, 701, 702]).unwrap();
    assert_eq!(
        line_signature(top_line),
        (
            (4, 2, EndpointReference::Vertex(41)),
            (1, -1, EndpointReference::Vertex(40)),
            FacetEdgeType::Top,
        )
    );

    let bottom = [[1.9, -1.9, 1.0], [4.9, 2.9, 1.0], [0.0, 0.0, 2.0]];
    assert_eq!(
        intersect_facet(1.0, &bottom, [40, 41, 42], [700, 701, 702]),
        None
    );

    let horizontal = [[1.0, 0.0, 1.0], [0.0, 1.0, 1.0], [0.0, 0.0, 1.0]];
    assert_eq!(
        intersect_facet(1.0, &horizontal, [40, 41, 42], [700, 701, 702]),
        None
    );
}

#[test]
fn task22b_facet_rounding_preserves_zero_length_lines() {
    let vertices = [[0.0, 0.0, -1.0], [0.4, 0.0, 1.0], [0.0, 0.4, 1.0]];
    let line = intersect_facet(0.0, &vertices, [0, 1, 2], [800, 801, 802]).unwrap();

    assert_eq!(
        line_signature(line),
        (
            (0, 0, EndpointReference::Edge(802)),
            (0, 0, EndpointReference::Edge(800)),
            FacetEdgeType::General,
        )
    );
    assert_eq!(line.a().point(), line.b().point());
}
