use std::{cell::RefCell, rc::Rc};

use crate::{
    arachne::skeletal::{
        EdgeId, NodeId, SkeletalEdge, SkeletalGraph, SkeletalJoint, TransitionEnd,
    },
    geometry::{CoordinateScale, Point},
};

use super::super::{
    SkeletalTrapezoidation,
    test_support::{config, strategy},
};

fn edge_pair(
    graph: &mut SkeletalGraph,
    from: NodeId,
    to: NodeId,
    central: bool,
) -> (EdgeId, EdgeId) {
    let edge = graph.add_edge(SkeletalEdge::default());
    let twin = graph.add_edge(SkeletalEdge::default());
    graph.edge_mut(edge).from = Some(from);
    graph.edge_mut(edge).to = Some(to);
    graph.edge_mut(twin).from = Some(to);
    graph.edge_mut(twin).to = Some(from);
    graph.edge_mut(edge).data.set_is_central(central);
    graph.edge_mut(twin).data.set_is_central(central);
    graph.connect_twins(edge, twin);
    (edge, twin)
}

#[test]
fn task22o172_applies_transition_end_as_central_node() {
    let scale = CoordinateScale::Normal;
    let scaled = |value| scale.checked_scale(value).unwrap();
    let strategy = strategy(scale);
    let mut graph = SkeletalGraph::default();
    let lower_left = graph.add_node(SkeletalJoint::default(), Point::new(0, 0));
    let lower_right = graph.add_node(SkeletalJoint::default(), Point::new(scaled(1.0), 0));
    let upper_left = graph.add_node(SkeletalJoint::default(), Point::new(0, scaled(1.0)));
    let upper_right = graph.add_node(
        SkeletalJoint::default(),
        Point::new(scaled(1.0), scaled(1.0)),
    );
    let from = graph.add_node(SkeletalJoint::default(), Point::new(0, scaled(0.5)));
    let to = graph.add_node(
        SkeletalJoint::default(),
        Point::new(scaled(1.0), scaled(0.5)),
    );
    graph.node_mut(from).data.distance_to_boundary = scaled(0.5);
    graph.node_mut(from).data.bead_count = 1;
    graph.node_mut(to).data.distance_to_boundary = scaled(0.5);
    graph.node_mut(to).data.bead_count = 2;
    let (edge, twin) = edge_pair(&mut graph, from, to, true);
    let (lower_in, _) = edge_pair(&mut graph, lower_left, from, false);
    let (lower_out, _) = edge_pair(&mut graph, to, lower_right, false);
    let (upper_in, _) = edge_pair(&mut graph, upper_right, to, false);
    let (upper_out, _) = edge_pair(&mut graph, from, upper_left, false);
    graph.edge_mut(lower_in).next = Some(edge);
    graph.edge_mut(edge).prev = Some(lower_in);
    graph.edge_mut(edge).next = Some(lower_out);
    graph.edge_mut(lower_out).prev = Some(edge);
    graph.edge_mut(upper_in).next = Some(twin);
    graph.edge_mut(twin).prev = Some(upper_in);
    graph.edge_mut(twin).next = Some(upper_out);
    graph.edge_mut(upper_out).prev = Some(twin);

    let end_storage = Rc::new(RefCell::new(vec![TransitionEnd::new(
        scaled(0.5),
        1,
        false,
    )]));
    graph.edge_mut(edge).data.set_transition_ends(&end_storage);
    let mut trapezoidation = SkeletalTrapezoidation {
        graph,
        beading_strategy: strategy.as_ref(),
        config: config(scale),
        vd_edge_to_he_edge: Default::default(),
        vd_node_to_he_node: Default::default(),
        transition_storage: Vec::new(),
        transition_end_storage: vec![end_storage],
    };

    trapezoidation.apply_transitions();

    let middle = Point::new(scaled(0.5), scaled(0.5));
    let inserted = trapezoidation
        .graph
        .active_nodes()
        .filter(|&node| trapezoidation.graph.node(node).point == middle)
        .collect::<Vec<_>>();
    assert_eq!(inserted.len(), 1);
    assert_eq!(trapezoidation.graph.node(inserted[0]).data.bead_count, 2);
}
