use super::*;
use crate::{
    arachne::skeletal::{SkeletalEdge, SkeletalGraph, SkeletalJoint, TransitionMiddle},
    geometry::{CoordinateScale, Point},
};

use super::super::test_support::{central_chain, config, strategy};

#[test]
fn task22o170_continues_transition_end_onto_next_central_edge() {
    let scale = CoordinateScale::Normal;
    let scaled = |value| scale.checked_scale(value).unwrap();
    let strategy = strategy(scale);
    let (mut graph, first, nodes) = central_chain(scale);
    let second = graph.edge(first).next.unwrap();
    graph.node_mut(nodes[0]).data.distance_to_boundary = scaled(0.2);
    graph.node_mut(nodes[1]).data.distance_to_boundary = scaled(0.4);
    graph.node_mut(nodes[2]).data.distance_to_boundary = scaled(0.6);
    let middle_position = scaled(0.9);
    let lower_bead_count = 1;
    let middle_storage = Rc::new(RefCell::new(vec![TransitionMiddle::new(
        middle_position,
        lower_bead_count,
        scaled(0.4),
    )]));
    graph.edge_mut(first).data.set_transitions(&middle_storage);
    let mut trapezoidation = SkeletalTrapezoidation {
        graph,
        beading_strategy: strategy.as_ref(),
        config: config(scale),
        vd_edge_to_he_edge: Default::default(),
        vd_node_to_he_node: Default::default(),
        transition_storage: vec![middle_storage],
        transition_end_storage: Vec::new(),
        beading_storage: Vec::new(),
        extrusion_junction_storage: Vec::new(),
        generated_toolpaths: Vec::new(),
    };

    trapezoidation.generate_all_transition_ends();

    let transition_length = strategy.transitioning_length(i64::from(lower_bead_count));
    let anchor = f64::from(strategy.transition_anchor_pos(i64::from(lower_bead_count)));
    let upper_half_length = ((1.0 - anchor) * transition_length as f64) as i64;
    let expected_position = middle_position + upper_half_length - scaled(1.0);
    assert_eq!(
        *trapezoidation
            .graph
            .edge(second)
            .data
            .transition_ends()
            .unwrap()
            .borrow(),
        vec![TransitionEnd::new(
            expected_position,
            lower_bead_count,
            false,
        )]
    );
    assert_eq!(
        trapezoidation.graph.node(nodes[1]).data.bead_count,
        i64::from(lower_bead_count)
    );
    assert!(trapezoidation.graph.node(nodes[1]).data.transition_ratio > anchor as f32);
}

#[test]
fn task22o171_skips_descending_branch_for_increasing_transition() {
    let scale = CoordinateScale::Normal;
    let scaled = |value| scale.checked_scale(value).unwrap();
    let strategy = strategy(scale);
    let mut graph = SkeletalGraph::default();
    let root = graph.add_node(SkeletalJoint::default(), Point::new(0, 0));
    let junction = graph.add_node(SkeletalJoint::default(), Point::new(scaled(0.1), 0));
    let upward = graph.add_node(SkeletalJoint::default(), Point::new(scaled(1.1), 0));
    let downward = graph.add_node(
        SkeletalJoint::default(),
        Point::new(scaled(0.1), scaled(1.0)),
    );
    for (node, radius, bead_count) in [
        (root, scaled(0.2), 1),
        (junction, scaled(0.4), 2),
        (upward, scaled(0.6), 2),
        (downward, 0, 1),
    ] {
        graph.node_mut(node).data.distance_to_boundary = radius;
        graph.node_mut(node).data.bead_count = bead_count;
    }
    let mut add_pair = |from, to| {
        let edge = graph.add_edge(SkeletalEdge::default());
        let twin = graph.add_edge(SkeletalEdge::default());
        graph.edge_mut(edge).from = Some(from);
        graph.edge_mut(edge).to = Some(to);
        graph.edge_mut(twin).from = Some(to);
        graph.edge_mut(twin).to = Some(from);
        graph.edge_mut(edge).data.set_is_central(true);
        graph.edge_mut(twin).data.set_is_central(true);
        graph.connect_twins(edge, twin);
        (edge, twin)
    };
    let (first, first_twin) = add_pair(root, junction);
    let (up, up_twin) = add_pair(junction, upward);
    let (down, down_twin) = add_pair(junction, downward);
    graph.edge_mut(first).next = Some(up);
    graph.edge_mut(up_twin).next = Some(down);
    graph.edge_mut(down_twin).next = Some(first_twin);
    graph.edge_mut(up).next = Some(up_twin);
    graph.edge_mut(down).next = Some(down_twin);

    let middle_position = scaled(0.09);
    let lower_bead_count = 1;
    let middle_storage = Rc::new(RefCell::new(vec![TransitionMiddle::new(
        middle_position,
        lower_bead_count,
        scaled(0.4),
    )]));
    graph.edge_mut(first).data.set_transitions(&middle_storage);
    let mut trapezoidation = SkeletalTrapezoidation {
        graph,
        beading_strategy: strategy.as_ref(),
        config: config(scale),
        vd_edge_to_he_edge: Default::default(),
        vd_node_to_he_node: Default::default(),
        transition_storage: vec![middle_storage],
        transition_end_storage: Vec::new(),
        beading_storage: Vec::new(),
        extrusion_junction_storage: Vec::new(),
        generated_toolpaths: Vec::new(),
    };

    trapezoidation.generate_all_transition_ends();

    assert!(
        trapezoidation
            .graph
            .edge(up)
            .data
            .has_transition_ends(false)
    );
    assert!(
        !trapezoidation
            .graph
            .edge(down_twin)
            .data
            .has_transition_ends(false)
    );
}
