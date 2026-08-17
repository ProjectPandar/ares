use std::{cell::RefCell, rc::Rc};

use crate::{
    arachne::{
        beading::base::Beading,
        extrusion_line::ExtrusionJunction,
        skeletal::{BeadingPropagation, SkeletalEdge, SkeletalGraph, SkeletalJoint},
    },
    geometry::{CoordinateScale, Point},
};

use super::super::super::{
    SkeletalTrapezoidation,
    test_support::{config, strategy},
};

#[test]
fn task22o183_generates_segment_junction_with_width_and_perimeter_index() {
    let scale = CoordinateScale::Normal;
    let strategy = strategy(scale);
    let mut graph = SkeletalGraph::default();
    let lower = graph.add_node(SkeletalJoint::default(), Point::new(0, 0));
    let upper = graph.add_node(SkeletalJoint::default(), Point::new(100_000, 0));
    graph.node_mut(lower).data.distance_to_boundary = 10_000;
    graph.node_mut(upper).data.distance_to_boundary = 40_000;
    graph.node_mut(upper).data.bead_count = 3;
    let edge = graph.add_edge(SkeletalEdge::default());
    let twin = graph.add_edge(SkeletalEdge::default());
    graph.edge_mut(edge).from = Some(lower);
    graph.edge_mut(edge).to = Some(upper);
    graph.edge_mut(twin).from = Some(upper);
    graph.edge_mut(twin).to = Some(lower);
    graph.connect_twins(edge, twin);
    let beading_storage = Rc::new(RefCell::new(BeadingPropagation::new(Beading {
        total_thickness: 80_000,
        bead_widths: vec![100, 200, 300],
        toolpath_locations: vec![5_000, 20_000, 35_000],
        left_over: 20_000,
    })));
    graph.node_mut(upper).data.set_beading(&beading_storage);
    let mut trapezoidation = SkeletalTrapezoidation {
        graph,
        beading_strategy: strategy.as_ref(),
        config: config(scale),
        vd_edge_to_he_edge: Default::default(),
        vd_node_to_he_node: Default::default(),
        transition_storage: Vec::new(),
        transition_end_storage: Vec::new(),
        beading_storage: vec![beading_storage],
        extrusion_junction_storage: Vec::new(),
        generated_toolpaths: Vec::new(),
    };

    trapezoidation.generate_junctions();

    assert_eq!(
        *trapezoidation
            .graph
            .edge(edge)
            .data
            .extrusion_junctions()
            .unwrap()
            .borrow(),
        vec![ExtrusionJunction::new(Point::new(33_334, 0), 200, 1)]
    );
}
