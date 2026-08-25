use std::{cell::RefCell, rc::Rc};

use crate::{
    arachne::{
        beading::base::Beading,
        skeletal::{BeadingPropagation, SkeletalEdge, SkeletalGraph, SkeletalJoint},
    },
    geometry::{CoordinateScale, Point},
};

use super::super::super::{
    SkeletalTrapezoidation,
    test_support::{config, strategy},
};

#[test]
fn task22o192_generates_six_segment_odd_ring_at_local_maximum() {
    let scale = CoordinateScale::Normal;
    let strategy = strategy(scale);
    let mut graph = SkeletalGraph::default();
    let peak = graph.add_node(SkeletalJoint::default(), Point::new(100, 100));
    let lower = graph.add_node(SkeletalJoint::default(), Point::new(200, 100));
    graph.node_mut(peak).data.distance_to_boundary = 50;
    graph.node_mut(lower).data.distance_to_boundary = 0;
    let outgoing = graph.add_edge(SkeletalEdge::default());
    let incoming = graph.add_edge(SkeletalEdge::default());
    graph.edge_mut(outgoing).from = Some(peak);
    graph.edge_mut(outgoing).to = Some(lower);
    graph.edge_mut(incoming).from = Some(lower);
    graph.edge_mut(incoming).to = Some(peak);
    graph.edge_mut(outgoing).data.set_is_central(false);
    graph.edge_mut(incoming).data.set_is_central(false);
    graph.connect_twins(outgoing, incoming);
    graph.edge_mut(incoming).next = Some(outgoing);
    graph.node_mut(peak).incident_edge = Some(outgoing);
    graph.node_mut(lower).incident_edge = Some(incoming);
    let beading_storage = Rc::new(RefCell::new(BeadingPropagation::new(Beading {
        total_thickness: 100,
        bead_widths: vec![40, 80, 40],
        toolpath_locations: vec![20, 50, 80],
        left_over: 0,
    })));
    graph.node_mut(peak).data.set_beading(&beading_storage);
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

    trapezoidation.generate_local_maxima_single_beads();

    let line = &trapezoidation.generated_toolpaths[1][0];
    assert!(line.is_odd);
    assert_eq!(line.junctions.len(), 6);
    assert_eq!(line.junctions[0].point, Point::new(110, 100));
    assert_eq!(
        line.junctions
            .iter()
            .map(|junction| junction.point)
            .collect::<Vec<_>>(),
        [
            (110, 100),
            (105, 109),
            (95, 109),
            (90, 100),
            (95, 91),
            (105, 91),
        ]
        .map(|(x, y)| Point::new(x, y)),
    );
    assert!(
        line.junctions
            .iter()
            .all(|junction| junction.width == 80 && junction.perimeter_index == 1)
    );
}

#[test]
fn task22o193_segment_stage_propagates_beading_before_local_maximum_ring() {
    let scale = CoordinateScale::Normal;
    let scaled = |value| scale.checked_scale(value).unwrap();
    let strategy = strategy(scale);
    let mut graph = SkeletalGraph::default();
    let peak = graph.add_node(SkeletalJoint::default(), Point::new(0, 0));
    let lower = graph.add_node(SkeletalJoint::default(), Point::new(scaled(2.0), 0));
    graph.node_mut(peak).data.distance_to_boundary = scaled(0.6);
    graph.node_mut(peak).data.bead_count = 3;
    graph.node_mut(lower).data.distance_to_boundary = 0;
    let outgoing = graph.add_edge(SkeletalEdge::default());
    let incoming = graph.add_edge(SkeletalEdge::default());
    graph.edge_mut(outgoing).from = Some(peak);
    graph.edge_mut(outgoing).to = Some(lower);
    graph.edge_mut(incoming).from = Some(lower);
    graph.edge_mut(incoming).to = Some(peak);
    graph.edge_mut(outgoing).data.set_is_central(false);
    graph.edge_mut(incoming).data.set_is_central(false);
    graph.connect_twins(outgoing, incoming);
    graph.edge_mut(outgoing).next = Some(incoming);
    graph.edge_mut(outgoing).prev = Some(incoming);
    graph.edge_mut(incoming).next = Some(outgoing);
    graph.edge_mut(incoming).prev = Some(outgoing);
    graph.node_mut(peak).incident_edge = Some(outgoing);
    graph.node_mut(lower).incident_edge = Some(incoming);
    let mut trapezoidation = SkeletalTrapezoidation {
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

    trapezoidation.generate_segments();

    assert!(trapezoidation.graph.node(peak).data.has_beading());
    let beading = trapezoidation.graph.node(peak).data.beading().unwrap();
    assert_eq!(beading.borrow().beading.bead_widths.len(), 3);
    let lines = trapezoidation
        .generated_toolpaths
        .iter()
        .flatten()
        .filter(|line| line.is_odd && line.junctions.len() == 6)
        .count();
    assert_eq!(
        lines, 1,
        "toolpaths: {:?}",
        trapezoidation.generated_toolpaths
    );
}
