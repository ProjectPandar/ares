use crate::{
    arachne::{
        beading::base::{Beading, BeadingStrategy, BeadingStrategyConfig},
        skeletal::{SkeletalGraph, SkeletalJoint},
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
