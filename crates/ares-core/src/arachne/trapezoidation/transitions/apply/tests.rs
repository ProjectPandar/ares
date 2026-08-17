use std::{cell::RefCell, rc::Rc};

use crate::{
    arachne::skeletal::TransitionEnd,
    geometry::{CoordinateScale, Point},
};

use super::super::{
    SkeletalTrapezoidation,
    test_support::{central_cell, config, strategy},
};

#[test]
fn task22o172_applies_transition_end_as_central_node() {
    let scale = CoordinateScale::Normal;
    let scaled = |value| scale.checked_scale(value).unwrap();
    let strategy = strategy(scale);
    let (mut graph, edge, _, from, to) = central_cell(scale);
    graph.node_mut(from).data.distance_to_boundary = scaled(0.5);
    graph.node_mut(from).data.bead_count = 1;
    graph.node_mut(to).data.distance_to_boundary = scaled(0.5);
    graph.node_mut(to).data.bead_count = 2;
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
        beading_storage: Vec::new(),
        extrusion_junction_storage: Vec::new(),
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
