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
    test_support::{central_cell, config, strategy},
};

use super::super::toolpaths::SegmentConditions;

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

#[test]
fn task22o185_connects_paired_junctions_from_inner_to_outer() {
    let scale = CoordinateScale::Normal;
    let strategy = strategy(scale);
    let mut graph = SkeletalGraph::default();
    let from_edge = graph.add_edge(SkeletalEdge::default());
    let to_edge = graph.add_edge(SkeletalEdge::default());
    let from_outer = ExtrusionJunction::new(Point::new(0, 0), 100, 0);
    let from_inner = ExtrusionJunction::new(Point::new(0, 10), 80, 1);
    let to_outer = ExtrusionJunction::new(Point::new(100, 0), 100, 0);
    let to_inner = ExtrusionJunction::new(Point::new(100, 10), 80, 1);
    let from_storage = Rc::new(RefCell::new(vec![from_outer, from_inner]));
    let to_storage = Rc::new(RefCell::new(vec![to_outer, to_inner]));
    graph
        .edge_mut(from_edge)
        .data
        .set_extrusion_junctions(&from_storage);
    graph
        .edge_mut(to_edge)
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
    let conditions = SegmentConditions {
        is_odd: false,
        force_new_path: false,
        from_is_three_way: false,
        to_is_three_way: false,
    };

    trapezoidation.connect_junction_pair(from_edge, to_edge, conditions);

    assert_eq!(
        trapezoidation.generated_toolpaths[0][0].junctions,
        vec![from_outer, to_outer]
    );
    assert_eq!(
        trapezoidation.generated_toolpaths[1][0].junctions,
        vec![from_inner, to_inner]
    );
}

#[test]
fn task22o186_selects_edge_entering_quad_maximum_radius() {
    let scale = CoordinateScale::Normal;
    let strategy = strategy(scale);
    let (mut graph, peak_edge, _twin, from, to) = central_cell(scale);
    let quad_start = graph.edge(peak_edge).prev.unwrap();
    let quad_end = graph.edge(peak_edge).next.unwrap();
    let start_boundary = graph.edge(quad_start).from.unwrap();
    let end_boundary = graph.edge(quad_end).to.unwrap();
    graph.node_mut(start_boundary).data.distance_to_boundary = 0;
    graph.node_mut(from).data.distance_to_boundary = 10_000;
    graph.node_mut(to).data.distance_to_boundary = 20_000;
    graph.node_mut(end_boundary).data.distance_to_boundary = 0;
    let trapezoidation = SkeletalTrapezoidation {
        graph,
        beading_strategy: strategy.as_ref(),
        config: config(scale),
        vd_edge_to_he_edge: Default::default(),
        vd_node_to_he_node: Default::default(),
        transition_storage: Vec::new(),
        transition_end_storage: Vec::new(),
        beading_storage: Vec::new(),
        extrusion_junction_storage: Vec::new(),
        generated_toolpaths: Vec::new(),
    };

    assert_eq!(trapezoidation.get_quad_max_r_edge_to(quad_start), peak_edge);
}
