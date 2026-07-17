use std::collections::{BTreeMap, BTreeSet};

use crate::mesh_slicer::{EndpointReference, IntersectionLine, IntersectionPoint};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct NodeKey {
    x: i64,
    y: i64,
    reference_kind: u8,
    reference_id: u32,
}

#[derive(Default)]
struct Incidence {
    incoming: usize,
    outgoing: usize,
    neighbors: Vec<NodeKey>,
}

pub(super) fn count(lines: &[IntersectionLine]) -> usize {
    let mut graph = BTreeMap::<NodeKey, Incidence>::new();
    for line in lines {
        let a = node_key(line.a());
        let b = node_key(line.b());
        let a_incidence = graph.entry(a).or_default();
        a_incidence.outgoing += 1;
        a_incidence.neighbors.push(b);
        let b_incidence = graph.entry(b).or_default();
        b_incidence.incoming += 1;
        b_incidence.neighbors.push(a);
    }

    let mut visited = BTreeSet::new();
    let mut closed = 0;
    for start in graph.keys().copied() {
        if !visited.insert(start) {
            continue;
        }
        closed += usize::from(component_is_closed(start, &graph, &mut visited));
    }
    closed
}

fn component_is_closed(
    start: NodeKey,
    graph: &BTreeMap<NodeKey, Incidence>,
    visited: &mut BTreeSet<NodeKey>,
) -> bool {
    let mut stack = vec![start];
    let mut component_closed = true;
    while let Some(node) = stack.pop() {
        let incidence = &graph[&node];
        component_closed &= incidence.incoming == 1 && incidence.outgoing == 1;
        for neighbor in incidence.neighbors.iter().copied() {
            if visited.insert(neighbor) {
                stack.push(neighbor);
            }
        }
    }
    component_closed
}

fn node_key(endpoint: IntersectionPoint) -> NodeKey {
    let point = endpoint.point();
    let (reference_kind, reference_id) = match endpoint.reference() {
        EndpointReference::Vertex(id) => (0, id),
        EndpointReference::Edge(id) => (1, id),
    };
    NodeKey {
        x: point.x(),
        y: point.y(),
        reference_kind,
        reference_id,
    }
}
