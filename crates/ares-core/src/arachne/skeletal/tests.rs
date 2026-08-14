mod edge_cases;

use std::{cell::RefCell, rc::Rc};

use crate::geometry::Point;

use super::{
    BeadingPropagation, EdgeId, EdgeType, NodeId, Shared, SkeletalEdge, SkeletalGraph,
    SkeletalJoint, TransitionEnd, TransitionMiddle,
};
use crate::arachne::{beading::base::Beading, extrusion_line::ExtrusionJunction};

fn node(graph: &mut SkeletalGraph, x: i64, y: i64, distance: i64) -> NodeId {
    let id = graph.add_node(SkeletalJoint::default(), Point::new(x, y));
    graph.node_mut(id).data.distance_to_boundary = distance;
    id
}

fn directed_edge(graph: &mut SkeletalGraph, from: NodeId, to: NodeId) -> EdgeId {
    let edge = graph.add_edge(SkeletalEdge::default());
    graph.edge_mut(edge).from = Some(from);
    graph.edge_mut(edge).to = Some(to);
    edge
}

fn edge_pair(graph: &mut SkeletalGraph, left: NodeId, right: NodeId) -> (EdgeId, EdgeId) {
    let forth = directed_edge(graph, left, right);
    let back = directed_edge(graph, right, left);
    graph.connect_twins(forth, back);
    (forth, back)
}

#[test]
fn task22o100_payloads_preserve_weak_shared_storage_and_source_defaults() {
    let mut edge = SkeletalEdge::new(EdgeType::ExtraVoronoi);
    assert_eq!(edge.edge_type, EdgeType::ExtraVoronoi);
    assert!(!edge.central_is_set());
    edge.set_is_central(true);
    assert!(edge.is_central());

    let transitions: Shared<_> = Rc::new(RefCell::new(vec![TransitionMiddle::new(7, 2, 11)]));
    let ends: Shared<_> = Rc::new(RefCell::new(vec![TransitionEnd::new(3, 1, true)]));
    let junctions: Shared<_> = Rc::new(RefCell::new(vec![ExtrusionJunction::new(
        Point::new(1, 2),
        40,
        3,
    )]));
    edge.set_transitions(&transitions);
    edge.set_transition_ends(&ends);
    edge.set_extrusion_junctions(&junctions);
    let transition = transitions.borrow()[0];
    assert_eq!(
        (
            transition.pos,
            transition.lower_bead_count,
            transition.feature_radius
        ),
        (7, 2, 11)
    );
    let end = ends.borrow()[0];
    assert_eq!(
        (end.pos, end.lower_bead_count, end.is_lower_end),
        (3, 1, true)
    );
    assert!(edge.has_transitions(false));
    assert!(edge.has_transition_ends(false));
    assert!(edge.has_extrusion_junctions(false));
    transitions.borrow_mut().clear();
    assert!(!edge.has_transitions(false));
    assert!(edge.has_transitions(true));
    drop(ends);
    assert!(!edge.has_transition_ends(true));

    let propagation = Rc::new(RefCell::new(BeadingPropagation::new(Beading::empty(90))));
    let mut joint = SkeletalJoint::default();
    assert_eq!(joint.distance_to_boundary, -1);
    assert_eq!(joint.bead_count, -1);
    assert_eq!(joint.transition_ratio, 0.0);
    joint.set_beading(&propagation);
    assert!(joint.has_beading());
    {
        let storage = joint.beading().unwrap();
        let stored = storage.borrow();
        assert_eq!(stored.beading.total_thickness, 90);
        assert_eq!(stored.dist_to_bottom_source, 0);
        assert_eq!(stored.dist_from_top_source, 0);
        assert!(!stored.is_upward_propagated_only);
    }
    drop(propagation);
    assert!(!joint.has_beading());
}

#[test]
fn task22o100_twin_cycles_incident_edges_and_local_maxima_match_source_walks() {
    let mut graph = SkeletalGraph::default();
    let center = node(&mut graph, 0, 0, 20);
    let outer = [
        node(&mut graph, 10, 0, 10),
        node(&mut graph, 0, 10, 10),
        node(&mut graph, -10, 0, 10),
    ];
    let pairs = outer.map(|outer| edge_pair(&mut graph, center, outer));
    for (forth, _) in pairs {
        graph.edge_mut(forth).data.set_is_central(true);
    }
    for (index, (_, twin)) in pairs.iter().enumerate() {
        graph.edge_mut(*twin).next = Some(pairs[(index + 1) % pairs.len()].0);
    }
    graph.node_mut(center).incident_edge = Some(pairs[0].0);

    assert_eq!(graph.edge(pairs[0].0).twin, Some(pairs[0].1));
    assert_eq!(graph.edge(pairs[0].1).twin, Some(pairs[0].0));
    assert!(graph.node_is_multi_intersection(center));
    assert!(graph.node_is_central(center));
    assert!(graph.node_is_local_maximum(center, false));
    assert!(!graph.edge_can_go_up(pairs[0].0, false));
    assert!(graph.edge_can_go_up(pairs[0].1, false));
}

#[test]
fn task22o100_upward_distance_ties_and_unconnected_chain_end_are_deterministic() {
    let mut graph = SkeletalGraph::default();
    let low = node(&mut graph, 0, 0, 0);
    let high = node(&mut graph, 3, 4, 8);
    let (up, down) = edge_pair(&mut graph, low, high);
    assert!(graph.edge_is_upward(up));
    assert!(!graph.edge_is_upward(down));
    assert_eq!(graph.edge_dist_to_go_up(up), Some(0));
    assert_eq!(graph.edge_dist_to_go_up(down), None);

    let equal_left = node(&mut graph, 10, 0, 4);
    let equal_right = node(&mut graph, 20, 0, 4);
    let (equal, equal_twin) = edge_pair(&mut graph, equal_left, equal_right);
    assert!(!graph.edge_is_upward(equal));
    assert!(graph.edge_is_upward(equal_twin));

    let chain_end = directed_edge(&mut graph, high, equal_left);
    let return_edge = directed_edge(&mut graph, equal_left, high);
    graph.edge_mut(up).next = Some(chain_end);
    graph.edge_mut(chain_end).twin = Some(return_edge);
    assert_eq!(graph.next_unconnected(up), Some(return_edge));
    graph.edge_mut(chain_end).next = Some(up);
    assert_eq!(graph.next_unconnected(up), None);
}

#[test]
fn task22o100_insert_node_splits_both_half_edges_and_adds_transition_ribs() {
    let mut graph = SkeletalGraph::default();
    let left = node(&mut graph, -10, 0, 0);
    let right = node(&mut graph, 10, 0, 0);
    let (edge, twin) = edge_pair(&mut graph, left, right);
    let replacement = graph.insert_node(edge, Point::new(0, 5), 3);

    assert_eq!(graph.active_nodes().count(), 5);
    assert_eq!(graph.active_edges().count(), 8);
    let middle = graph.edge(replacement).from.unwrap();
    assert_eq!(graph.node(middle).point, Point::new(0, 5));
    assert_eq!(graph.node(middle).data.distance_to_boundary, 5);
    assert_eq!(graph.node(middle).data.bead_count, 3);
    assert_eq!(graph.edge(replacement).to, Some(right));
    assert_eq!(graph.edge(edge).to, Some(middle));
    assert_eq!(graph.edge(graph.edge(edge).twin.unwrap()).to, Some(left));
    assert_eq!(
        graph.edge(graph.edge(replacement).twin.unwrap()).from,
        Some(right)
    );
    assert!(graph.edge(edge).data.is_central());
    assert!(graph.edge(replacement).data.is_central());
    assert_ne!(graph.edge(twin).twin, Some(edge));
    let rib = graph.edge(edge).next.unwrap();
    assert_eq!(graph.edge(rib).data.edge_type, EdgeType::TransitionEnd);
    assert!(!graph.edge(rib).data.is_central());
    assert_eq!(graph.source(edge).a, Point::new(-10, 0));
    assert_eq!(graph.source(edge).b, Point::new(0, 0));
}

#[test]
fn task22o100_make_rib_projects_to_infinite_source_and_links_the_chain() {
    let mut graph = SkeletalGraph::default();
    let from = node(&mut graph, 0, 0, 0);
    let target = node(&mut graph, 20, 10, -1);
    let previous = directed_edge(&mut graph, from, target);
    let back = graph.make_rib(previous, Point::new(0, 0), Point::new(10, 0));
    let forth = graph.edge(back).twin.unwrap();
    let source = graph.edge(back).from.unwrap();
    assert_eq!(graph.node(source).point, Point::new(20, 0));
    assert_eq!(graph.node(target).data.distance_to_boundary, 10);
    assert_eq!(graph.edge(previous).next, Some(forth));
    assert_eq!(graph.edge(forth).prev, Some(previous));
    assert_eq!(graph.node(source).incident_edge, Some(back));
    assert_eq!(graph.active_nodes().next(), Some(source));
    assert_eq!(
        graph.active_edges().take(3).collect::<Vec<_>>(),
        vec![back, forth, previous]
    );
}

#[test]
fn task22o100_middle_collapse_rewires_both_cells_and_keeps_stable_ids() {
    let mut graph = SkeletalGraph::default();
    let a = node(&mut graph, 0, 0, 0);
    let b = node(&mut graph, 10, 0, 2);
    let c = node(&mut graph, 12, 0, 2);
    let d = node(&mut graph, 100, 0, 0);
    let (start, start_twin) = edge_pair(&mut graph, a, b);
    let (middle, middle_twin) = edge_pair(&mut graph, b, c);
    let (end, end_twin) = edge_pair(&mut graph, c, d);
    graph.edge_mut(start).next = Some(middle);
    graph.edge_mut(middle).prev = Some(start);
    graph.edge_mut(middle).next = Some(end);
    graph.edge_mut(end).prev = Some(middle);
    graph.edge_mut(end_twin).next = Some(middle_twin);
    graph.edge_mut(middle_twin).prev = Some(end_twin);
    graph.edge_mut(middle_twin).next = Some(start_twin);
    graph.edge_mut(start_twin).prev = Some(middle_twin);
    graph.node_mut(b).incident_edge = Some(middle);

    graph.collapse_small_edges(2);
    assert!(!graph.contains_node(c));
    assert!(!graph.contains_edge(middle));
    assert!(!graph.contains_edge(middle_twin));
    assert_eq!(graph.edge(start).next, Some(end));
    assert_eq!(graph.edge(end).prev, Some(start));
    assert_eq!(graph.edge(end).from, Some(b));
    assert_eq!(graph.edge(end_twin).to, Some(b));
    assert_eq!(graph.node(b).incident_edge, Some(start_twin));
    let new_node = node(&mut graph, 200, 0, 0);
    let new_edge = directed_edge(&mut graph, d, new_node);
    assert!(new_node.0 > c.0);
    assert!(new_edge.0 > middle_twin.0);
}

#[test]
fn task22o100_side_collapse_removes_the_cell_and_retwins_survivors() {
    let mut graph = SkeletalGraph::default();
    let a = node(&mut graph, 0, 0, 0);
    let b = node(&mut graph, 0, 10, 2);
    let c = node(&mut graph, 1, 10, 2);
    let d = node(&mut graph, 1, 0, 0);
    let (start, start_twin) = edge_pair(&mut graph, a, b);
    let (end, end_twin) = edge_pair(&mut graph, c, d);
    graph.edge_mut(start).next = Some(end);
    graph.edge_mut(end).prev = Some(start);
    graph.node_mut(c).incident_edge = Some(end);

    graph.collapse_small_edges(2);
    assert!(!graph.contains_node(a));
    assert!(!graph.contains_edge(start));
    assert!(!graph.contains_edge(end));
    assert_eq!(graph.edge(start_twin).twin, Some(end_twin));
    assert_eq!(graph.edge(end_twin).twin, Some(start_twin));
    assert_eq!(graph.edge(start_twin).to, Some(d));
    assert_eq!(graph.node(d).incident_edge, Some(end_twin));
    assert_eq!(graph.node(c).incident_edge, Some(start_twin));
}

#[test]
fn task22o100_removal_leaves_holes_without_aliasing_or_reusing_identity() {
    let mut graph = SkeletalGraph::default();
    let first_node = node(&mut graph, 0, 0, 0);
    let first_edge = graph.add_edge(SkeletalEdge::default());
    assert!(!graph.node_is_central(first_node));
    assert!(!graph.node_is_multi_intersection(first_node));
    assert!(!graph.node_is_local_maximum(first_node, false));
    assert_eq!(graph.next_unconnected(first_edge), None);
    graph.remove_node(first_node);
    graph.remove_edge(first_edge);
    let second_node = node(&mut graph, 0, 0, 0);
    let second_edge = graph.add_edge(SkeletalEdge::default());
    assert_eq!(graph.active_nodes().collect::<Vec<_>>(), vec![second_node]);
    assert_eq!(graph.active_edges().collect::<Vec<_>>(), vec![second_edge]);
    assert_ne!(first_node, second_node);
    assert_ne!(first_edge, second_edge);
}

#[test]
fn task22o100_clamped_projection_returns_exact_large_integer_endpoint() {
    let mut graph = SkeletalGraph::default();
    let start_x = (1_i64 << 53) + 1;
    let end_x = start_x + 100;
    let left = node(&mut graph, start_x, 0, 0);
    let right = node(&mut graph, end_x, 0, 0);
    let (edge, _) = edge_pair(&mut graph, left, right);

    graph.insert_node(edge, Point::new(end_x + 100, 5), 3);

    assert_eq!(
        graph
            .active_nodes()
            .filter(|&node| graph.node(node).point == Point::new(end_x, 0))
            .count(),
        3
    );
}
