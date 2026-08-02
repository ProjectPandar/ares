use crate::geometry::{Point, Polygon};
use crate::project_slice::perimeters::classic::traversal::{
    LowerFlowRoute, TraversalSeed,
    types::{PendingExtrusionRole, PendingLoopRole},
};

use super::consume_seeds;

fn seed(id: i64) -> TraversalSeed {
    TraversalSeed {
        polygon: Polygon::new(vec![
            Point::new(id, 0),
            Point::new(id + 1, 0),
            Point::new(id, 1),
        ]),
        depth: 1,
        is_contour: false,
        is_smaller_width_perimeter: false,
        extrusion_role: PendingExtrusionRole::Perimeter,
        loop_role: PendingLoopRole::Hole,
        route: LowerFlowRoute::Internal,
        width: 0.4,
        mm3_per_mm: 0.08,
        children: Vec::new(),
    }
}

#[test]
fn task22o5_terminal_seed_sink_is_iterative_on_a_constrained_stack() {
    std::thread::Builder::new()
        .stack_size(64 * 1024)
        .spawn(|| {
            let mut root = seed(20_000);
            for id in (0..20_000).rev() {
                let mut parent = seed(id);
                parent.children.push(root);
                root = parent;
            }
            consume_seeds(vec![root]);
        })
        .unwrap()
        .join()
        .unwrap();
}
