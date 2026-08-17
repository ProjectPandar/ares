use crate::{
    arachne::{
        beading::{
            base::BeadingStrategy,
            factory::{BeadingStrategyFactoryConfig, make_strategy},
        },
        skeletal::{EdgeId, NodeId, SkeletalEdge, SkeletalGraph, SkeletalJoint},
    },
    geometry::{CoordinateScale, Point},
};

use super::super::TrapezoidationConfig;

pub(super) fn strategy(scale: CoordinateScale) -> Box<dyn BeadingStrategy> {
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

pub(super) fn config(scale: CoordinateScale) -> TrapezoidationConfig {
    let scaled = |value| scale.checked_scale(value).unwrap();
    TrapezoidationConfig {
        transitioning_angle: 1.0,
        discretization_step_size: scaled(0.1),
        transition_filter_dist: scaled(0.4),
        allowed_filter_deviation: scaled(0.02),
        beading_propagation_transition_dist: scaled(0.4),
        coordinate_scale: scale,
    }
}

pub(super) fn central_chain(scale: CoordinateScale) -> (SkeletalGraph, EdgeId, [NodeId; 3]) {
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

pub(super) fn central_cell(
    scale: CoordinateScale,
) -> (SkeletalGraph, EdgeId, EdgeId, NodeId, NodeId) {
    let scaled = |value| scale.checked_scale(value).unwrap();
    let mut graph = SkeletalGraph::default();
    let lower_left = graph.add_node(SkeletalJoint::default(), Point::new(0, 0));
    let lower_right = graph.add_node(SkeletalJoint::default(), Point::new(scaled(1.0), 0));
    let upper_left = graph.add_node(SkeletalJoint::default(), Point::new(0, scaled(1.0)));
    let upper_right = graph.add_node(
        SkeletalJoint::default(),
        Point::new(scaled(1.0), scaled(1.0)),
    );
    let from = graph.add_node(SkeletalJoint::default(), Point::new(0, scaled(0.5)));
    let to = graph.add_node(
        SkeletalJoint::default(),
        Point::new(scaled(1.0), scaled(0.5)),
    );
    let (edge, twin) = edge_pair(&mut graph, from, to, true);
    let (lower_in, _) = edge_pair(&mut graph, lower_left, from, false);
    let (lower_out, _) = edge_pair(&mut graph, to, lower_right, false);
    let (upper_in, _) = edge_pair(&mut graph, upper_right, to, false);
    let (upper_out, _) = edge_pair(&mut graph, from, upper_left, false);
    graph.edge_mut(lower_in).next = Some(edge);
    graph.edge_mut(edge).prev = Some(lower_in);
    graph.edge_mut(edge).next = Some(lower_out);
    graph.edge_mut(lower_out).prev = Some(edge);
    graph.edge_mut(upper_in).next = Some(twin);
    graph.edge_mut(twin).prev = Some(upper_in);
    graph.edge_mut(twin).next = Some(upper_out);
    graph.edge_mut(upper_out).prev = Some(twin);
    (graph, edge, twin, from, to)
}

fn edge_pair(
    graph: &mut SkeletalGraph,
    from: NodeId,
    to: NodeId,
    central: bool,
) -> (EdgeId, EdgeId) {
    let edge = graph.add_edge(SkeletalEdge::default());
    let twin = graph.add_edge(SkeletalEdge::default());
    graph.edge_mut(edge).from = Some(from);
    graph.edge_mut(edge).to = Some(to);
    graph.edge_mut(twin).from = Some(to);
    graph.edge_mut(twin).to = Some(from);
    graph.edge_mut(edge).data.set_is_central(central);
    graph.edge_mut(twin).data.set_is_central(central);
    graph.connect_twins(edge, twin);
    (edge, twin)
}
