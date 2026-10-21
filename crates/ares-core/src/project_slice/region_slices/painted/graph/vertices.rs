//! MultiMaterialSegmentation.cpp::MMU_Graph::append_voronoi_vertices202–279.

use super::{
    Graph, Node,
    math::*,
    point_lookup::{Lookup, Sample},
};
use crate::geometry::voronoi::Diagram;

pub(super) fn append(
    graph: &mut Graph,
    diagram: &Diagram,
    bbox: Bounds,
    epsilon: (i64, f64),
) -> Vec<usize> {
    // BoundingBox::offset(double) → Point.hpp:197, distinct from lookup radius casts.
    let bbox = bbox.inflated(epsilon.1.round() as i64);
    let mut vertices = Lookup::new(epsilon.0);
    let mut contours = Lookup::new(3 * epsilon.0);
    for (contour, (&offset, &count)) in graph.offsets.iter().zip(&graph.sizes).enumerate() {
        for index in 0..count {
            contours.insert(Sample {
                exact: graph.nodes[offset + index].point,
                contour,
                index,
            });
        }
    }
    let mut mapping = Vec::with_capacity(diagram.num_vertices());
    for vertex in diagram.vertices() {
        let edge = vertex.get_incident_edge().unwrap();
        let twin = diagram.edge_get_twin(edge).unwrap();
        let source = diagram
            .cell(diagram.edge_get_cell(edge).unwrap())
            .unwrap()
            .source_index()
            .usize();
        let other = diagram
            .cell(diagram.edge_get_cell(twin).unwrap())
            .unwrap()
            .source_index()
            .usize();
        let a = graph.nodes[graph.arcs[source].from].point;
        let b = graph.nodes[graph.arcs[other].from].point;
        let exact = [vertex.x(), vertex.y()];
        let query = truncated(exact);
        let index = if ulp_equal(exact, a) {
            graph.arcs[source].from
        } else if ulp_equal(exact, b) {
            graph.arcs[other].from
        } else if bbox.contains(query) {
            node_for_point(graph, &mut vertices, &contours, exact, epsilon)
        } else {
            usize::MAX
        };
        mapping.push(index);
    }
    mapping
}

fn node_for_point(
    graph: &mut Graph,
    vertices: &mut Lookup,
    contours: &Lookup,
    exact: [f64; 2],
    epsilon: (i64, f64),
) -> usize {
    let query = truncated(exact);
    if let Some((p, distance)) = contours.find(query)
        && distance < (3.0 * epsilon.1).powi(2)
    {
        return graph.offsets[p.contour] + p.index;
    }
    if let Some((_, distance)) = vertices.find(query)
        && distance < (epsilon.1 / 10.0).powi(2)
    {
        for (p, _) in vertices.find_all(query) {
            let delta = sub(exact, p.exact);
            if dot(delta, delta) <= 1e-4_f64.powi(2) {
                return p.index;
            }
        }
    }
    let index = graph.nodes.len();
    vertices.insert(Sample {
        exact,
        contour: 0,
        index,
    });
    graph.nodes.push(Node {
        point: exact,
        arcs: Vec::new(),
    });
    index
}
