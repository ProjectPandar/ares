use crate::{
    arachne::{
        beading::factory::{BeadingStrategyFactoryConfig, make_strategy},
        skeletal::{EdgeId, SkeletalEdge, SkeletalGraph, SkeletalJoint},
    },
    geometry::{CoordinateScale, Point, Polygon},
};

use super::{
    SkeletalTrapezoidation, TrapezoidationConfig,
    discretize::{discretize_parabola, discretize_point_point},
    index::PolygonSegmentIndex,
};

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

fn config(scale: CoordinateScale) -> TrapezoidationConfig {
    TrapezoidationConfig {
        transitioning_angle: 1.0,
        discretization_step_size: scale.checked_scale(0.1).unwrap(),
        transition_filter_dist: scale.checked_scale(0.4).unwrap(),
        allowed_filter_deviation: scale.checked_scale(0.02).unwrap(),
        beading_propagation_transition_dist: scale.checked_scale(0.4).unwrap(),
        coordinate_scale: scale,
    }
}

fn rectangle(size: i64) -> Polygon {
    Polygon::new(vec![
        Point::new(0, 0),
        Point::new(size, 0),
        Point::new(size, size),
        Point::new(0, size),
    ])
}

fn node(
    graph: &mut SkeletalGraph,
    point: Point,
    distance: i64,
) -> crate::arachne::skeletal::NodeId {
    let node = graph.add_node(SkeletalJoint::default(), point);
    graph.node_mut(node).data.distance_to_boundary = distance;
    node
}

fn pair(
    graph: &mut SkeletalGraph,
    from: crate::arachne::skeletal::NodeId,
    to: crate::arachne::skeletal::NodeId,
) -> (EdgeId, EdgeId) {
    let edge = graph.add_edge(SkeletalEdge::default());
    let twin = graph.add_edge(SkeletalEdge::default());
    graph.edge_mut(edge).from = Some(from);
    graph.edge_mut(edge).to = Some(to);
    graph.edge_mut(twin).from = Some(to);
    graph.edge_mut(twin).to = Some(from);
    graph.connect_twins(edge, twin);
    (edge, twin)
}

fn empty<'a>(
    strategy: &'a dyn crate::arachne::beading::base::BeadingStrategy,
    scale: CoordinateScale,
) -> SkeletalTrapezoidation<'a> {
    SkeletalTrapezoidation {
        graph: SkeletalGraph::default(),
        beading_strategy: strategy,
        config: config(scale),
        vd_edge_to_he_edge: Default::default(),
        vd_node_to_he_node: Default::default(),
        transition_storage: Vec::new(),
        transition_end_storage: Vec::new(),
    }
}

#[test]
fn task22o101_rectangle_builds_stable_twinned_graph_at_both_scales() {
    for scale in [CoordinateScale::Normal, CoordinateScale::LargeBed] {
        let strategy = strategy(scale);
        let size = scale.checked_scale(10.0).unwrap();
        let mut trapezoidation =
            SkeletalTrapezoidation::new(&[rectangle(size)], strategy.as_ref(), config(scale))
                .unwrap();
        assert!(trapezoidation.graph.active_nodes().count() > 4);
        assert!(trapezoidation.graph.active_edges().count() > 8);
        for edge in trapezoidation.graph.active_edges() {
            let twin = trapezoidation.graph.edge(edge).twin.unwrap();
            assert_eq!(trapezoidation.graph.edge(twin).twin, Some(edge));
            assert_eq!(
                trapezoidation.graph.edge(edge).from,
                trapezoidation.graph.edge(twin).to
            );
            assert_eq!(
                trapezoidation.graph.edge(edge).to,
                trapezoidation.graph.edge(twin).from
            );
        }
        assert!(
            trapezoidation
                .graph
                .active_nodes()
                .any(|node| trapezoidation.graph.node(node).data.distance_to_boundary == 0)
        );
        trapezoidation.update_is_central();
        trapezoidation.filter_central(scale.checked_scale(0.02).unwrap());
        if matches!(scale, CoordinateScale::LargeBed) {
            trapezoidation.filter_outer_central();
        }
        trapezoidation.update_bead_count();
        trapezoidation.filter_noncentral_regions();
        assert!(
            trapezoidation
                .graph
                .active_edges()
                .all(|edge| trapezoidation.graph.edge(edge).data.central_is_set())
        );
    }
}

#[test]
fn task22o101_hole_preserves_polygon_segment_site_topology() {
    let scale = CoordinateScale::LargeBed;
    let strategy = strategy(scale);
    let unit = scale.checked_scale(1.0).unwrap();
    let hole = Polygon::new(vec![
        Point::new(3 * unit, 3 * unit),
        Point::new(3 * unit, 7 * unit),
        Point::new(7 * unit, 7 * unit),
        Point::new(7 * unit, 3 * unit),
    ]);
    let trapezoidation = SkeletalTrapezoidation::new(
        &[rectangle(10 * unit), hole],
        strategy.as_ref(),
        config(scale),
    )
    .unwrap();
    let boundary_nodes = trapezoidation
        .graph
        .active_nodes()
        .filter(|&node| trapezoidation.graph.node(node).data.distance_to_boundary == 0)
        .count();
    assert!(boundary_nodes >= 8);
}

#[test]
fn task22o101_point_segment_and_point_point_curves_use_source_rounding() {
    let polygons = vec![rectangle(1_000)];
    let parabola = discretize_parabola(
        Point::new(500, 500),
        PolygonSegmentIndex {
            polygon_index: 0,
            point_index: 0,
        },
        &polygons,
        Point::new(100, 420),
        Point::new(900, 420),
        100,
        1.0,
    );
    assert_eq!(parabola.first(), Some(&Point::new(100, 420)));
    assert_eq!(parabola.last(), Some(&Point::new(900, 420)));
    assert!(parabola.len() > 4);

    let point_point = discretize_point_point(
        Point::new(0, 0),
        Point::new(1_000, 0),
        Point::new(500, -1_000),
        Point::new(500, 1_000),
        200,
        1.0,
    );
    assert_eq!(point_point.first(), Some(&Point::new(500, -1_000)));
    assert_eq!(point_point.last(), Some(&Point::new(500, 1_000)));
    assert!(point_point.contains(&Point::new(500, 0)));
}

#[test]
fn task22o101_pointy_quad_start_is_split_without_reusing_identity() {
    let strategy = strategy(CoordinateScale::Normal);
    let mut trapezoidation = empty(strategy.as_ref(), CoordinateScale::Normal);
    let shared = node(&mut trapezoidation.graph, Point::new(0, 0), 0);
    let left = node(&mut trapezoidation.graph, Point::new(-10, 10), 1);
    let right = node(&mut trapezoidation.graph, Point::new(10, 10), 1);
    let (first, first_twin) = pair(&mut trapezoidation.graph, shared, left);
    let (second, second_twin) = pair(&mut trapezoidation.graph, shared, right);
    trapezoidation.graph.edge_mut(first_twin).prev = Some(first_twin);
    trapezoidation.graph.edge_mut(second_twin).prev = Some(second_twin);

    trapezoidation.separate_pointy_quad_end_nodes().unwrap();

    let first_from = trapezoidation.graph.edge(first).from.unwrap();
    let second_from = trapezoidation.graph.edge(second).from.unwrap();
    assert_ne!(first_from, second_from);
    assert_eq!(
        trapezoidation.graph.node(first_from).point,
        Point::new(0, 0)
    );
    assert_eq!(
        trapezoidation.graph.node(second_from).point,
        Point::new(0, 0)
    );
    assert_eq!(trapezoidation.graph.edge(second_twin).to, Some(second_from));
}

#[test]
fn task22o101_central_recursion_keeps_branch_that_reaches_local_maximum() {
    let strategy = strategy(CoordinateScale::Normal);
    let mut trapezoidation = empty(strategy.as_ref(), CoordinateScale::Normal);
    let a = node(&mut trapezoidation.graph, Point::new(0, 0), 5);
    let b = node(&mut trapezoidation.graph, Point::new(10, 0), 5);
    let c = node(&mut trapezoidation.graph, Point::new(20, 0), 8);
    let (equal, equal_twin) = pair(&mut trapezoidation.graph, a, b);
    let (up, up_twin) = pair(&mut trapezoidation.graph, b, c);
    for edge in [equal, equal_twin, up, up_twin] {
        trapezoidation
            .graph
            .edge_mut(edge)
            .data
            .set_is_central(true);
    }
    trapezoidation.graph.edge_mut(equal).next = Some(up);
    trapezoidation.graph.edge_mut(up_twin).next = Some(equal_twin);
    trapezoidation.graph.edge_mut(up).next = Some(up_twin);
    trapezoidation.graph.node_mut(c).incident_edge = Some(up_twin);

    assert!(!trapezoidation.filter_central_from(equal, 0, 100));
    assert!(trapezoidation.graph.edge(equal).data.is_central());
    assert!(trapezoidation.graph.edge(up).data.is_central());
}

#[test]
fn task22o101_noncentral_filter_connects_equal_bead_regions() {
    let strategy = strategy(CoordinateScale::Normal);
    let mut trapezoidation = empty(strategy.as_ref(), CoordinateScale::Normal);
    let a = node(&mut trapezoidation.graph, Point::new(0, 0), 0);
    let b = node(&mut trapezoidation.graph, Point::new(10, 0), 10);
    let c = node(&mut trapezoidation.graph, Point::new(20, 0), 20);
    trapezoidation.graph.node_mut(b).data.bead_count = 1;
    trapezoidation.graph.node_mut(c).data.bead_count = 1;
    let (central, _) = pair(&mut trapezoidation.graph, a, b);
    let (bridge, _) = pair(&mut trapezoidation.graph, b, c);
    trapezoidation
        .graph
        .edge_mut(central)
        .data
        .set_is_central(true);
    trapezoidation
        .graph
        .edge_mut(bridge)
        .data
        .set_is_central(false);
    let bridge_twin = trapezoidation.graph.edge(bridge).twin.unwrap();
    trapezoidation
        .graph
        .edge_mut(bridge_twin)
        .data
        .set_is_central(false);
    trapezoidation.graph.edge_mut(central).next = Some(bridge);

    assert!(trapezoidation.filter_noncentral_from(central, 1, 0, 100));
    assert!(trapezoidation.graph.edge(bridge).data.is_central());
    assert_eq!(trapezoidation.graph.node(c).data.transition_ratio, 0.0);
}
