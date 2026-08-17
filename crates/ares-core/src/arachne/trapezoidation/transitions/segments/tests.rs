use crate::{
    arachne::skeletal::{SkeletalGraph, SkeletalJoint},
    geometry::{CoordinateScale, Point},
};

use super::super::{
    SkeletalTrapezoidation,
    test_support::{config, strategy},
};

#[test]
fn task22o175_stores_exact_strategy_beading_on_zero_transition_node() {
    let scale = CoordinateScale::Normal;
    let scaled = |value| scale.checked_scale(value).unwrap();
    let strategy = strategy(scale);
    let mut graph = SkeletalGraph::default();
    let node = graph.add_node(SkeletalJoint::default(), Point::new(0, 0));
    graph.node_mut(node).data.distance_to_boundary = scaled(0.5);
    graph.node_mut(node).data.bead_count = 2;
    graph.node_mut(node).data.transition_ratio = 0.0;
    let expected = strategy.compute(scaled(1.0), 2);
    let mut trapezoidation = SkeletalTrapezoidation {
        graph,
        beading_strategy: strategy.as_ref(),
        config: config(scale),
        vd_edge_to_he_edge: Default::default(),
        vd_node_to_he_node: Default::default(),
        transition_storage: Vec::new(),
        transition_end_storage: Vec::new(),
        beading_storage: Vec::new(),
    };

    trapezoidation.store_node_beadings();

    assert_eq!(
        trapezoidation
            .graph
            .node(node)
            .data
            .beading()
            .unwrap()
            .borrow()
            .beading,
        expected
    );
}
