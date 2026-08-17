use crate::geometry::{CoordinateScale, Point};

use super::super::{
    SkeletalTrapezoidation,
    test_support::{central_cell, config, strategy},
};

#[test]
fn task22o174_inserts_nonlinear_radius_crossing_as_extra_rib() {
    let scale = CoordinateScale::Normal;
    let scaled = |value| scale.checked_scale(value).unwrap();
    let strategy = strategy(scale);
    let thickness = *strategy
        .nonlinear_thicknesses(1)
        .first()
        .expect("thin-wall strategy must expose a nonlinear thickness");
    let target_radius = thickness / 2;
    let start_radius = target_radius / 2;
    let end_radius = target_radius + target_radius / 2;
    let (mut graph, edge, _, from, to) = central_cell(scale);
    graph.node_mut(from).data.distance_to_boundary = start_radius;
    graph.node_mut(from).data.bead_count = 1;
    graph.node_mut(to).data.distance_to_boundary = end_radius;
    graph.node_mut(to).data.bead_count = 2;
    let mut trapezoidation = SkeletalTrapezoidation {
        graph,
        beading_strategy: strategy.as_ref(),
        config: config(scale),
        vd_edge_to_he_edge: Default::default(),
        vd_node_to_he_node: Default::default(),
        transition_storage: Vec::new(),
        transition_end_storage: Vec::new(),
    };

    trapezoidation.generate_extra_ribs();

    let edge_length = scaled(1.0);
    let expected_x = edge_length * (target_radius - start_radius) / (end_radius - start_radius);
    let expected = Point::new(expected_x, scaled(0.5));
    let inserted = trapezoidation
        .graph
        .active_nodes()
        .filter(|&node| trapezoidation.graph.node(node).point == expected)
        .collect::<Vec<_>>();
    assert_eq!(inserted.len(), 1);
    assert_eq!(trapezoidation.graph.node(inserted[0]).data.bead_count, 1);
    assert!(trapezoidation.graph.edge(edge).data.is_central());
}
