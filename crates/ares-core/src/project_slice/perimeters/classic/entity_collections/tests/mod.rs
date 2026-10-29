use super::{
    super::{
        chained_loops::{ChainedLoopNode, ExtrusionLoop, ExtrusionLoopRole},
        materialize::{ExtrusionPath, ExtrusionRole, Point3, Polyline3},
        shortest_path::{
            chain_extrusion_entities, chain_extrusion_loops, chain_loops_by_first_points,
        },
        traversal::{LowerFlowRoute, PendingExtrusionRole, PendingLoopRole, TraversalSeed},
    },
    orientation::{is_counter_clockwise, orient_loop, reverse_loop},
    traverse::traverse_loops,
    types::ExtrusionEntity,
};
use crate::{
    ProcessWallDirection,
    geometry::{CoordinateScale, Point, Polygon, Polyline, to_thick_polylines},
};

fn test_flow() -> crate::project_slice::perimeters::types::Flow {
    crate::project_slice::perimeters::types::Flow {
        width: 0.45,
        height: 0.2,
        spacing: 0.45,
        nozzle_diameter: 0.4,
        bridge: false,
        mm3_per_mm: 0.09,
    }
}
const TEST_SCALE: CoordinateScale = CoordinateScale::Normal;

/// Test seam for the materialization boundary: convert the thin walls once
/// and chain the root level exactly as `materialize/tree.rs` does, so
/// `traverse_loops` receives production-shaped `(thin_entities, root_order)`
/// inputs instead of recomputing anything itself.
fn root_thin_entities_and_order(
    nodes: &[ChainedLoopNode],
    thin_walls: &[crate::geometry::ThickPolyline],
) -> (Vec<ExtrusionEntity>, Vec<(usize, bool)>) {
    let thin_entities =
        crate::project_slice::perimeters::classic::gap_extrusion::variable_width::convert_entities(
            thin_walls,
            test_flow(),
            TEST_SCALE,
            ExtrusionRole::ExternalPerimeter,
        )
        .unwrap_or_default();
    let loops: Vec<&ExtrusionLoop> = nodes
        .iter()
        .filter_map(|node| node.extrusion_loop.as_ref())
        .collect();
    let order = if thin_entities.is_empty() {
        let first_points: Vec<_> = loops
            .iter()
            .map(|loop_| {
                let first = loop_.paths[0].polyline.points[0];
                [first.x as f64, first.y as f64]
            })
            .collect();
        chain_loops_by_first_points(&first_points)
            .into_iter()
            .map(|index| (index, false))
            .collect()
    } else {
        let mut endpoints = Vec::with_capacity(loops.len() + thin_entities.len());
        let mut is_loop = Vec::with_capacity(loops.len() + thin_entities.len());
        for loop_ in &loops {
            let first = loop_.paths[0].polyline.points[0];
            let last = loop_
                .paths
                .last()
                .and_then(|path| path.polyline.points.last())
                .copied()
                .unwrap_or(first);
            endpoints.push([[first.x, first.y], [last.x, last.y]]);
            is_loop.push(true);
        }
        for entity in &thin_entities {
            endpoints.push(entity_endpoints(entity));
            is_loop.push(matches!(entity, ExtrusionEntity::Loop(_)));
        }
        chain_extrusion_entities(&endpoints, &is_loop, [0, 0])
    };
    (thin_entities, order)
}

fn entity_endpoints(entity: &ExtrusionEntity) -> [[i64; 2]; 2] {
    let points_of =
        |first: crate::project_slice::perimeters::classic::materialize::Point3,
         last: crate::project_slice::perimeters::classic::materialize::Point3| {
            [[first.x, first.y], [last.x, last.y]]
        };
    match entity {
        ExtrusionEntity::Loop(ordered) => {
            let first = ordered.extrusion_loop.paths[0].polyline.points[0];
            let last = ordered
                .extrusion_loop
                .paths
                .last()
                .and_then(|p| p.polyline.points.last())
                .copied()
                .unwrap_or(first);
            points_of(first, last)
        }
        ExtrusionEntity::Path(path) => {
            let first = path.polyline.points[0];
            let last = path.polyline.points.last().copied().unwrap_or(first);
            points_of(first, last)
        }
        ExtrusionEntity::MultiPath(multi) => {
            let first = multi.paths[0].polyline.points[0];
            let last = multi
                .paths
                .last()
                .and_then(|p| p.polyline.points.last())
                .copied()
                .unwrap_or(first);
            points_of(first, last)
        }
    }
}

fn deep_pair(depth: u16) -> (ChainedLoopNode, TraversalSeed) {
    let mut node = ChainedLoopNode {
        extrusion_loop: Some(square_loop(1, 0, ExtrusionLoopRole::Default)),
        children: Vec::new(),
        child_order: Vec::new(),
    };
    let mut root_seed = seed(true, 0, Vec::new());
    for depth in 1..depth {
        node = ChainedLoopNode {
            extrusion_loop: Some(square_loop(
                i64::from(depth) + 1,
                0,
                ExtrusionLoopRole::Default,
            )),
            children: vec![node],
            // Single-child level: its chain is the trivial self order.
            child_order: vec![(0, false)],
        };
        root_seed = seed(true, depth, vec![root_seed]);
    }
    (node, root_seed)
}

fn square_loop(x: i64, y: i64, role: ExtrusionLoopRole) -> ExtrusionLoop {
    ExtrusionLoop {
        paths: vec![path(
            &[(x, y), (x + 4, y), (x + 4, y + 4), (x, y + 4), (x, y)],
            ExtrusionRole::Perimeter,
        )],
        role,
    }
}

fn path(points: &[(i64, i64)], role: ExtrusionRole) -> ExtrusionPath {
    ExtrusionPath {
        z_contoured: false,
        polyline: Polyline3 {
            points: points.iter().map(|&(x, y)| Point3 { x, y, z: 0 }).collect(),
            fitting: Vec::new(),
        },
        role,
        can_reverse: true,
        mm3_per_mm: 1.0,
        width: 0.4,
        height: 0.2,
    }
}

fn seed(is_contour: bool, depth: u16, children: Vec<TraversalSeed>) -> TraversalSeed {
    TraversalSeed {
        polygon: Polygon::new(vec![Point::new(0, 0), Point::new(4, 0), Point::new(4, 4)]),
        depth,
        is_contour,
        is_smaller_width_perimeter: false,
        extrusion_role: PendingExtrusionRole::Perimeter,
        loop_role: if is_contour {
            PendingLoopRole::Default
        } else {
            PendingLoopRole::Hole
        },
        route: LowerFlowRoute::Internal,
        width: 0.4,
        mm3_per_mm: 1.0,
        children,
    }
}

fn drain_seeds(mut seeds: Vec<TraversalSeed>) {
    while let Some(mut seed) = seeds.pop() {
        seeds.append(&mut seed.children);
    }
}

fn thin_wall(points: &[(i64, i64)]) -> crate::geometry::ThickPolyline {
    to_thick_polylines(
        vec![Polyline::new(
            points.iter().map(|&(x, y)| Point::new(x, y)).collect(),
        )],
        0.4e6,
    )
    .remove(0)
}

mod loops;
mod thin_walls;
