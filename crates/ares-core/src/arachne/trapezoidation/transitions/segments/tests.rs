use crate::{
    arachne::{
        beading::base::{Beading, BeadingStrategy, BeadingStrategyConfig},
        skeletal::{EdgeId, SkeletalEdge, SkeletalGraph, SkeletalJoint},
    },
    geometry::{CoordinateScale, Point},
};

use super::super::{
    SkeletalTrapezoidation,
    test_support::{config, strategy},
};

struct DeterministicTransitionStrategy {
    config: BeadingStrategyConfig,
}

impl BeadingStrategy for DeterministicTransitionStrategy {
    fn config(&self) -> &BeadingStrategyConfig {
        &self.config
    }

    fn compute(&self, thickness: i64, bead_count: i64) -> Beading {
        match bead_count {
            2 => Beading {
                total_thickness: thickness,
                bead_widths: vec![200, 400],
                toolpath_locations: vec![100, 500],
                left_over: 400,
            },
            3 => Beading {
                total_thickness: thickness,
                bead_widths: vec![100, 300, 500],
                toolpath_locations: vec![50, 300, 750],
                left_over: 100,
            },
            _ => unreachable!(),
        }
    }

    fn optimal_bead_count(&self, _thickness: i64) -> i64 {
        2
    }

    fn description(&self) -> String {
        "DeterministicTransitionStrategy".to_owned()
    }
}

fn upward_quad_candidate(
    graph: &mut SkeletalGraph,
    x: i64,
    from_radius: i64,
    to_radius: i64,
) -> EdgeId {
    let from = graph.add_node(SkeletalJoint::default(), Point::new(x, 0));
    let to = graph.add_node(SkeletalJoint::default(), Point::new(x + 100, 0));
    graph.node_mut(from).data.distance_to_boundary = from_radius;
    graph.node_mut(to).data.distance_to_boundary = to_radius;
    let edge = graph.add_edge(SkeletalEdge::default());
    let twin = graph.add_edge(SkeletalEdge::default());
    graph.edge_mut(edge).from = Some(from);
    graph.edge_mut(edge).to = Some(to);
    graph.edge_mut(twin).from = Some(to);
    graph.edge_mut(twin).to = Some(from);
    graph.edge_mut(edge).prev = Some(twin);
    graph.edge_mut(edge).next = Some(twin);
    graph.edge_mut(edge).data.set_is_central(true);
    graph.edge_mut(twin).data.set_is_central(true);
    graph.connect_twins(edge, twin);
    edge
}

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

#[test]
fn task22o176_interpolates_transitional_node_beading() {
    let scale = CoordinateScale::Normal;
    let strategy = DeterministicTransitionStrategy {
        config: BeadingStrategyConfig {
            optimal_width: 100,
            wall_split_middle_threshold: 0.5,
            wall_add_middle_threshold: 0.5,
            default_transition_length: 400,
            transitioning_angle: 1.0,
            coordinate_scale: scale,
        },
    };
    let mut graph = SkeletalGraph::default();
    let node = graph.add_node(SkeletalJoint::default(), Point::new(0, 0));
    graph.node_mut(node).data.distance_to_boundary = 500;
    graph.node_mut(node).data.bead_count = 2;
    graph.node_mut(node).data.transition_ratio = 0.25;
    let mut trapezoidation = SkeletalTrapezoidation {
        graph,
        beading_strategy: &strategy,
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
        Beading {
            total_thickness: 1_000,
            bead_widths: vec![175, 375, 500],
            toolpath_locations: vec![87, 450, 750],
            left_over: 100,
        }
    );
}

#[test]
fn task22o177_orders_upward_quad_mids_by_descending_radius() {
    let scale = CoordinateScale::Normal;
    let strategy = strategy(scale);
    let mut graph = SkeletalGraph::default();
    let lower = upward_quad_candidate(&mut graph, 0, 10, 20);
    let higher = upward_quad_candidate(&mut graph, 200, 20, 40);
    let trapezoidation = SkeletalTrapezoidation {
        graph,
        beading_strategy: strategy.as_ref(),
        config: config(scale),
        vd_edge_to_he_edge: Default::default(),
        vd_node_to_he_node: Default::default(),
        transition_storage: Vec::new(),
        transition_end_storage: Vec::new(),
        beading_storage: Vec::new(),
    };

    assert_eq!(trapezoidation.upward_quad_mids(), vec![higher, lower]);
}

#[test]
fn task22o178_propagates_lower_beading_to_unassigned_upper_node() {
    let scale = CoordinateScale::Normal;
    let strategy = strategy(scale);
    let mut graph = SkeletalGraph::default();
    let edge = upward_quad_candidate(&mut graph, 0, 10, 20);
    let from = graph.edge(edge).from.unwrap();
    let to = graph.edge(edge).to.unwrap();
    graph.node_mut(from).data.bead_count = 2;
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
    let expected = trapezoidation
        .graph
        .node(from)
        .data
        .beading()
        .unwrap()
        .borrow()
        .beading
        .clone();

    trapezoidation.propagate_beadings_upward(&[edge]);

    let upper_storage = trapezoidation.graph.node(to).data.beading().unwrap();
    let upper = upper_storage.borrow();
    assert_eq!(upper.beading, expected);
    assert_eq!(upper.dist_to_bottom_source, 100);
    assert_eq!(upper.dist_from_top_source, 0);
    assert!(upper.is_upward_propagated_only);
}

#[test]
fn task22o179_copies_peak_beading_to_empty_lower_node() {
    let scale = CoordinateScale::Normal;
    let strategy = strategy(scale);
    let mut graph = SkeletalGraph::default();
    let edge = upward_quad_candidate(&mut graph, 0, 10, 20);
    let from = graph.edge(edge).from.unwrap();
    let to = graph.edge(edge).to.unwrap();
    graph.node_mut(to).data.bead_count = 3;
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
    let expected = trapezoidation
        .graph
        .node(to)
        .data
        .beading()
        .unwrap()
        .borrow()
        .beading
        .clone();

    trapezoidation.propagate_beading_downward_to_empty_lower(edge);

    let lower_storage = trapezoidation.graph.node(from).data.beading().unwrap();
    let lower = lower_storage.borrow();
    assert_eq!(lower.beading, expected);
    assert_eq!(lower.dist_to_bottom_source, 0);
    assert_eq!(lower.dist_from_top_source, 100);
    assert!(!lower.is_upward_propagated_only);
}
