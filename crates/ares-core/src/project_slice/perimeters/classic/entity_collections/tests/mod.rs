use super::{
    super::{
        chained_loops::{ChainedLoopNode, ExtrusionLoop, ExtrusionLoopRole},
        materialize::{ExtrusionPath, ExtrusionRole, Point3, Polyline3},
        shortest_path::chain_extrusion_loops,
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

/// The unit traversal fixtures never produce thin walls, so `traverse_loops` is
/// exercised with an empty `thin_walls` slice and a placeholder
/// `ext_perimeter_flow`/`scale` (unused while `thin_walls.is_empty()`).
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

fn deep_pair(depth: u16) -> (ChainedLoopNode, TraversalSeed) {
    let mut node = ChainedLoopNode {
        extrusion_loop: Some(square_loop(1, 0, ExtrusionLoopRole::Default)),
        children: Vec::new(),
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
