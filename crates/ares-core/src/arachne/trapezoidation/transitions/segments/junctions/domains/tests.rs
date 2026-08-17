use std::{cell::RefCell, collections::HashSet, rc::Rc};

use crate::{
    arachne::{
        extrusion_line::ExtrusionJunction,
        skeletal::{SkeletalEdge, SkeletalGraph, SkeletalJoint},
    },
    geometry::{CoordinateScale, Point},
};

use super::super::super::super::{
    SkeletalTrapezoidation,
    test_support::{config, strategy},
};

#[test]
fn task22o190_walks_closed_domain_and_closes_one_toolpath() {
    let scale = CoordinateScale::Normal;
    let strategy = strategy(scale);
    let mut graph = SkeletalGraph::default();
    let left = graph.add_node(SkeletalJoint::default(), Point::new(0, 0));
    let peak = graph.add_node(SkeletalJoint::default(), Point::new(50, 20));
    let right = graph.add_node(SkeletalJoint::default(), Point::new(100, 0));
    graph.node_mut(left).data.distance_to_boundary = 0;
    graph.node_mut(peak).data.distance_to_boundary = 20;
    graph.node_mut(peak).data.bead_count = 2;
    graph.node_mut(right).data.distance_to_boundary = 0;
    let first_in = graph.add_edge(SkeletalEdge::default());
    let first_out = graph.add_edge(SkeletalEdge::default());
    let second_in = graph.add_edge(SkeletalEdge::default());
    let second_out = graph.add_edge(SkeletalEdge::default());
    for (edge, from, to) in [
        (first_in, left, peak),
        (first_out, peak, right),
        (second_in, right, peak),
        (second_out, peak, left),
    ] {
        graph.edge_mut(edge).from = Some(from);
        graph.edge_mut(edge).to = Some(to);
    }
    graph.connect_twins(first_in, second_out);
    graph.connect_twins(first_out, second_in);
    graph.edge_mut(first_in).next = Some(first_out);
    graph.edge_mut(first_out).prev = Some(first_in);
    graph.edge_mut(second_in).next = Some(second_out);
    graph.edge_mut(second_out).prev = Some(second_in);
    let first = ExtrusionJunction::new(Point::new(40, 10), 100, 0);
    let second = ExtrusionJunction::new(Point::new(60, 10), 100, 0);
    let first_storage = Rc::new(RefCell::new(vec![first]));
    let second_storage = Rc::new(RefCell::new(vec![second]));
    graph
        .edge_mut(first_in)
        .data
        .set_extrusion_junctions(&first_storage);
    graph
        .edge_mut(second_in)
        .data
        .set_extrusion_junctions(&second_storage);
    let mut trapezoidation = SkeletalTrapezoidation {
        graph,
        beading_strategy: strategy.as_ref(),
        config: config(scale),
        vd_edge_to_he_edge: Default::default(),
        vd_node_to_he_node: Default::default(),
        transition_storage: Vec::new(),
        transition_end_storage: Vec::new(),
        beading_storage: Vec::new(),
        extrusion_junction_storage: vec![first_storage, second_storage],
        generated_toolpaths: Vec::new(),
    };

    trapezoidation.connect_junctions();

    let line = &trapezoidation.generated_toolpaths[0][0];
    assert_eq!(line.junctions.len(), 3);
    assert_eq!(
        line.junctions.first().unwrap().point,
        line.junctions.last().unwrap().point
    );
    assert_ne!(line.junctions[0].point, line.junctions[1].point);
}

#[test]
fn task22o191_skips_odd_segment_when_opposite_edge_was_passed() {
    let scale = CoordinateScale::Normal;
    let strategy = strategy(scale);
    let mut graph = SkeletalGraph::default();
    let left = graph.add_node(SkeletalJoint::default(), Point::new(0, 0));
    let peak = graph.add_node(SkeletalJoint::default(), Point::new(50, 20));
    let right = graph.add_node(SkeletalJoint::default(), Point::new(100, 0));
    graph.node_mut(left).data.distance_to_boundary = 0;
    graph.node_mut(peak).data.distance_to_boundary = 20;
    graph.node_mut(peak).data.bead_count = 3;
    graph.node_mut(right).data.distance_to_boundary = 0;
    let incoming = graph.add_edge(SkeletalEdge::default());
    let incoming_twin = graph.add_edge(SkeletalEdge::default());
    let outgoing = graph.add_edge(SkeletalEdge::default());
    let outgoing_twin = graph.add_edge(SkeletalEdge::default());
    for (edge, from, to) in [
        (incoming, left, peak),
        (incoming_twin, peak, left),
        (outgoing, peak, right),
        (outgoing_twin, right, peak),
    ] {
        graph.edge_mut(edge).from = Some(from);
        graph.edge_mut(edge).to = Some(to);
    }
    graph.connect_twins(incoming, incoming_twin);
    graph.connect_twins(outgoing, outgoing_twin);
    graph.edge_mut(incoming).next = Some(outgoing);
    graph.edge_mut(outgoing).prev = Some(incoming);
    let from = ExtrusionJunction::new(Point::new(50, 20), 100, 0);
    let to = ExtrusionJunction::new(Point::new(50, 20), 101, 0);
    let from_storage = Rc::new(RefCell::new(vec![from]));
    let to_storage = Rc::new(RefCell::new(vec![to]));
    graph
        .edge_mut(incoming)
        .data
        .set_extrusion_junctions(&from_storage);
    graph
        .edge_mut(outgoing_twin)
        .data
        .set_extrusion_junctions(&to_storage);
    let mut trapezoidation = SkeletalTrapezoidation {
        graph,
        beading_strategy: strategy.as_ref(),
        config: config(scale),
        vd_edge_to_he_edge: Default::default(),
        vd_node_to_he_node: Default::default(),
        transition_storage: Vec::new(),
        transition_end_storage: Vec::new(),
        beading_storage: Vec::new(),
        extrusion_junction_storage: vec![from_storage, to_storage],
        generated_toolpaths: Vec::new(),
    };
    let mut passed = HashSet::from([outgoing_twin]);

    trapezoidation.connect_quad_junctions_with_passed(incoming, true, &mut passed);

    assert!(trapezoidation.generated_toolpaths.is_empty());
}
