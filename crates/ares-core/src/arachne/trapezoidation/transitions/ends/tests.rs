use super::*;
use crate::{arachne::skeletal::TransitionMiddle, geometry::CoordinateScale};

use super::super::tests::{central_chain, config, strategy};

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
