use super::super::{
    super::{
        materialize::{ExtrusionPath, ExtrusionRole, Point3, Polyline3, RawPathNode},
        traversal::{
            LowerFlowRoute, PendingExtrusionRole, PendingLoopRole, PendingPathBranch, TraversalSeed,
        },
    },
    tree::{consume_nodes, transform_nodes},
};
use crate::geometry::{Point, Polygon};

#[test]
fn task22o8_transform_and_sink_are_iterative_on_a_constrained_stack() {
    std::thread::Builder::new()
        .stack_size(64 * 1024)
        .spawn(|| {
            run_deep(PendingPathBranch::OrdinaryUnsplit {
                detect_overhang_wall: false,
                layer_id: 1,
                raft_layers: 0,
            });
            run_deep(PendingPathBranch::OverhangClipping {
                detect_overhang_wall: true,
                layer_id: 1,
                raft_layers: 0,
            });
        })
        .unwrap()
        .join()
        .unwrap();
}

fn run_deep(branch: PendingPathBranch) {
    let mut current_seed = seed(Vec::new());
    let mut raw_node = raw(Vec::new());
    for _ in 0..10_000 {
        current_seed = seed(vec![current_seed]);
        raw_node = raw(vec![raw_node]);
    }
    let output = transform_nodes(vec![raw_node], std::slice::from_ref(&current_seed), branch);
    assert_eq!(output.len(), 1);
    consume_nodes(output);
    consume_seeds(vec![current_seed]);
}

fn seed(children: Vec<TraversalSeed>) -> TraversalSeed {
    TraversalSeed {
        polygon: Polygon::new(vec![
            Point::new(0, 0),
            Point::new(10, 0),
            Point::new(10, 10),
        ]),
        depth: 0,
        is_contour: true,
        is_smaller_width_perimeter: false,
        extrusion_role: PendingExtrusionRole::Perimeter,
        loop_role: PendingLoopRole::Default,
        route: LowerFlowRoute::Internal,
        width: 0.4,
        mm3_per_mm: 1.0,
        children,
    }
}

fn raw(children: Vec<RawPathNode>) -> RawPathNode {
    RawPathNode {
        paths: vec![ExtrusionPath {
            polyline: Polyline3 {
                points: vec![Point3 { x: 0, y: 0, z: 0 }, Point3 { x: 10, y: 0, z: 0 }],
            },
            role: ExtrusionRole::Perimeter,
            mm3_per_mm: 1.0,
            width: 0.4,
            height: 0.2,
        }],
        children,
    }
}

fn consume_seeds(mut seeds: Vec<TraversalSeed>) {
    while let Some(mut seed) = seeds.pop() {
        let _ = (
            seed.polygon,
            seed.depth,
            seed.is_contour,
            seed.is_smaller_width_perimeter,
            seed.extrusion_role,
            seed.loop_role,
            seed.route,
            seed.width,
            seed.mm3_per_mm,
        );
        seeds.append(&mut seed.children);
    }
}
