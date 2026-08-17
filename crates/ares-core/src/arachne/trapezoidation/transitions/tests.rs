use super::*;
use crate::arachne::{
    beading::factory::{BeadingStrategyFactoryConfig, make_strategy},
    skeletal::{EdgeId, SkeletalEdge, SkeletalGraph, SkeletalJoint},
};
use crate::geometry::CoordinateScale;

fn strategy(scale: CoordinateScale) -> Box<dyn crate::arachne::beading::base::BeadingStrategy> {
    let scaled = |value| scale.checked_scale(value).unwrap();
    make_strategy(BeadingStrategyFactoryConfig {
        preferred_bead_width_outer: scaled(0.42),
        preferred_bead_width_inner: scaled(0.45),
        preferred_transition_length: scaled(0.4),
        transitioning_angle: 1.0,
        print_thin_walls: true,
        min_bead_width: scaled(0.2),
        min_feature_size: scaled(0.25),
        wall_split_middle_threshold: 0.5,
        wall_add_middle_threshold: 0.5,
        max_bead_count: 4,
        outer_wall_offset: 0,
        inward_distributed_center_wall_count: 1,
        minimum_variable_line_ratio: 0.8,
        coordinate_scale: scale,
    })
}

fn config(scale: CoordinateScale) -> super::super::TrapezoidationConfig {
    let scaled = |value| scale.checked_scale(value).unwrap();
    super::super::TrapezoidationConfig {
        transitioning_angle: 1.0,
        discretization_step_size: scaled(0.1),
        transition_filter_dist: scaled(0.4),
        allowed_filter_deviation: scaled(0.02),
        beading_propagation_transition_dist: scaled(0.4),
        coordinate_scale: scale,
    }
}

fn central_chain(
    scale: CoordinateScale,
) -> (SkeletalGraph, EdgeId, [crate::arachne::skeletal::NodeId; 3]) {
    let unit = scale.checked_scale(1.0).unwrap();
    let mut graph = SkeletalGraph::default();
    let nodes = [
        graph.add_node(SkeletalJoint::default(), Point::new(0, 0)),
        graph.add_node(SkeletalJoint::default(), Point::new(unit, 0)),
        graph.add_node(SkeletalJoint::default(), Point::new(2 * unit, 0)),
    ];
    for node in nodes {
        graph.node_mut(node).data.bead_count = 3;
    }
    let first = graph.add_edge(SkeletalEdge::default());
    let first_twin = graph.add_edge(SkeletalEdge::default());
    let second = graph.add_edge(SkeletalEdge::default());
    let second_twin = graph.add_edge(SkeletalEdge::default());
    for (edge, from, to) in [
        (first, nodes[0], nodes[1]),
        (first_twin, nodes[1], nodes[0]),
        (second, nodes[1], nodes[2]),
        (second_twin, nodes[2], nodes[1]),
    ] {
        graph.edge_mut(edge).from = Some(from);
        graph.edge_mut(edge).to = Some(to);
        graph.edge_mut(edge).data.set_is_central(true);
    }
    graph.connect_twins(first, first_twin);
    graph.connect_twins(second, second_twin);
    graph.edge_mut(first).next = Some(second);
    graph.edge_mut(second_twin).next = Some(first_twin);
    graph.edge_mut(second).next = Some(second_twin);
    (graph, first, nodes)
}

#[test]
fn task22o102_generates_ordered_mids_on_upward_central_edge() {
    let scale = CoordinateScale::Normal;
    let scaled = |value| scale.checked_scale(value).unwrap();
    let strategy = strategy(scale);
    let mut graph = SkeletalGraph::default();
    let from = graph.add_node(SkeletalJoint::default(), Point::new(0, 0));
    let to = graph.add_node(SkeletalJoint::default(), Point::new(scaled(2.0), 0));
    graph.node_mut(from).data.distance_to_boundary = scaled(0.3);
    graph.node_mut(to).data.distance_to_boundary = scaled(2.0);
    graph.node_mut(from).data.bead_count = 1;
    graph.node_mut(to).data.bead_count = 3;
    let edge = graph.add_edge(SkeletalEdge::default());
    let twin = graph.add_edge(SkeletalEdge::default());
    graph.edge_mut(edge).from = Some(from);
    graph.edge_mut(edge).to = Some(to);
    graph.edge_mut(twin).from = Some(to);
    graph.edge_mut(twin).to = Some(from);
    graph.connect_twins(edge, twin);
    graph.edge_mut(edge).data.set_is_central(true);
    graph.edge_mut(twin).data.set_is_central(true);
    let mut trapezoidation = SkeletalTrapezoidation {
        graph,
        beading_strategy: strategy.as_ref(),
        config: super::super::TrapezoidationConfig {
            transitioning_angle: 1.0,
            discretization_step_size: scaled(0.1),
            transition_filter_dist: scaled(0.4),
            allowed_filter_deviation: scaled(0.02),
            beading_propagation_transition_dist: scaled(0.4),
            coordinate_scale: scale,
        },
        vd_edge_to_he_edge: Default::default(),
        vd_node_to_he_node: Default::default(),
        transition_storage: Vec::new(),
    };

    trapezoidation.generate_transition_mids();

    let transitions = trapezoidation.graph.edge(edge).data.transitions().unwrap();
    let transitions = transitions.borrow();
    assert_eq!(transitions.len(), 2);
    assert!(transitions[0].pos < transitions[1].pos);
    assert_eq!(transitions[0].lower_bead_count, 1);
    assert_eq!(transitions[1].lower_bead_count, 2);
    let start_radius = scaled(0.3);
    let end_radius = scaled(2.0);
    let edge_length = scaled(2.0);
    for transition in transitions.iter() {
        let expected_radius =
            strategy.transition_thickness(i64::from(transition.lower_bead_count)) / 2;
        assert_eq!(transition.feature_radius, expected_radius);
        assert_eq!(
            transition.pos,
            edge_length * (expected_radius - start_radius) / (end_radius - start_radius)
        );
    }
}

#[test]
fn task22o164_replaces_bead_count_through_reached_central_terminal() {
    let scale = CoordinateScale::Normal;
    let strategy = strategy(scale);
    let (graph, edge, nodes) = central_chain(scale);
    let mut trapezoidation = SkeletalTrapezoidation {
        graph,
        beading_strategy: strategy.as_ref(),
        config: config(scale),
        vd_edge_to_he_edge: Default::default(),
        vd_node_to_he_node: Default::default(),
        transition_storage: Vec::new(),
    };

    assert!(trapezoidation.filter_end_of_central_transition(
        edge,
        0,
        scale.checked_scale(3.0).unwrap(),
        2,
    ));
    assert_eq!(trapezoidation.graph.node(nodes[1]).data.bead_count, 2);
    assert_eq!(trapezoidation.graph.node(nodes[2]).data.bead_count, 2);
}

#[test]
fn task22o164_keeps_bead_count_when_terminal_exceeds_limit() {
    let scale = CoordinateScale::Normal;
    let strategy = strategy(scale);
    let (graph, edge, nodes) = central_chain(scale);
    let mut trapezoidation = SkeletalTrapezoidation {
        graph,
        beading_strategy: strategy.as_ref(),
        config: config(scale),
        vd_edge_to_he_edge: Default::default(),
        vd_node_to_he_node: Default::default(),
        transition_storage: Vec::new(),
    };

    assert!(!trapezoidation.filter_end_of_central_transition(
        edge,
        0,
        scale.checked_scale(0.5).unwrap(),
        2,
    ));
    assert_eq!(trapezoidation.graph.node(nodes[1]).data.bead_count, 3);
    assert_eq!(trapezoidation.graph.node(nodes[2]).data.bead_count, 3);
}

#[test]
fn task22o165_replaces_only_matching_connected_central_region() {
    let scale = CoordinateScale::Normal;
    let strategy = strategy(scale);
    let mut graph = SkeletalGraph::default();
    let nodes = (0..5)
        .map(|index| {
            graph.add_node(
                SkeletalJoint::default(),
                Point::new(scale.checked_scale(index as f64).unwrap(), 0),
            )
        })
        .collect::<Vec<_>>();
    for (index, &node) in nodes.iter().enumerate() {
        graph.node_mut(node).data.bead_count = if index == 4 { 4 } else { 3 };
    }
    let mut pairs = Vec::new();
    for &target in &nodes[1..] {
        let edge = graph.add_edge(SkeletalEdge::default());
        let twin = graph.add_edge(SkeletalEdge::default());
        graph.edge_mut(edge).from = Some(nodes[0]);
        graph.edge_mut(edge).to = Some(target);
        graph.edge_mut(twin).from = Some(target);
        graph.edge_mut(twin).to = Some(nodes[0]);
        graph.connect_twins(edge, twin);
        graph.edge_mut(edge).data.set_is_central(true);
        graph.edge_mut(twin).data.set_is_central(true);
        pairs.push((edge, twin));
    }
    let (source, source_twin) = pairs[0];
    let (matching, matching_twin) = pairs[1];
    let (noncentral, noncentral_twin) = pairs[2];
    let (different, different_twin) = pairs[3];
    graph.edge_mut(noncentral).data.set_is_central(false);
    graph.edge_mut(noncentral_twin).data.set_is_central(false);
    graph.edge_mut(source).next = Some(matching);
    graph.edge_mut(matching_twin).next = Some(noncentral);
    graph.edge_mut(noncentral_twin).next = Some(different);
    graph.edge_mut(different_twin).next = Some(source_twin);
    graph.edge_mut(matching).next = Some(matching_twin);
    graph.edge_mut(different).next = Some(different_twin);
    let mut trapezoidation = SkeletalTrapezoidation {
        graph,
        beading_strategy: strategy.as_ref(),
        config: config(scale),
        vd_edge_to_he_edge: Default::default(),
        vd_node_to_he_node: Default::default(),
        transition_storage: Vec::new(),
    };

    trapezoidation.dissolve_bead_count_region(source, 3, 2);

    assert_eq!(trapezoidation.graph.node(nodes[1]).data.bead_count, 2);
    assert_eq!(trapezoidation.graph.node(nodes[2]).data.bead_count, 2);
    assert_eq!(trapezoidation.graph.node(nodes[3]).data.bead_count, 3);
    assert_eq!(trapezoidation.graph.node(nodes[4]).data.bead_count, 4);
}

#[test]
fn task22o166_discovers_matching_transition_only_inside_distance_limit() {
    let scale = CoordinateScale::Normal;
    let strategy = strategy(scale);
    let (mut graph, source, nodes) = central_chain(scale);
    let candidate = graph.edge(source).next.unwrap();
    let radius = scale.checked_scale(0.3).unwrap();
    graph.node_mut(nodes[1]).data.distance_to_boundary = radius;
    graph.node_mut(nodes[2]).data.distance_to_boundary = scale.checked_scale(0.5).unwrap();
    let position = scale.checked_scale(0.2).unwrap();
    let storage = std::rc::Rc::new(std::cell::RefCell::new(vec![TransitionMiddle::new(
        position, 1, radius,
    )]));
    graph.edge_mut(candidate).data.set_transitions(&storage);
    let trapezoidation = SkeletalTrapezoidation {
        graph,
        beading_strategy: strategy.as_ref(),
        config: config(scale),
        vd_edge_to_he_edge: Default::default(),
        vd_node_to_he_node: Default::default(),
        transition_storage: vec![storage],
    };
    let origin = TransitionMiddle::new(0, 1, radius);

    let nearby = trapezoidation.dissolve_nearby_transitions(
        source,
        0,
        NearbyTransitionSearch {
            origin,
            maximum_distance: scale.checked_scale(2.0).unwrap(),
            going_up: false,
        },
    );
    assert_eq!(
        nearby,
        [TransitionMidRef {
            edge: candidate,
            index: 0
        }]
    );
    assert!(
        trapezoidation
            .dissolve_nearby_transitions(
                source,
                0,
                NearbyTransitionSearch {
                    origin,
                    maximum_distance: scale.checked_scale(0.1).unwrap(),
                    going_up: false,
                },
            )
            .is_empty()
    );
}
