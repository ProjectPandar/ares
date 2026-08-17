use super::super::{
    super::{
        materialize::{ExtrusionPath, ExtrusionRole, Point3, Polyline3, RawPathNode},
        traversal::{
            LowerFlowRoute, PendingExtrusionRole, PendingLoopRole, PendingPathBranch, TraversalSeed,
        },
    },
    ExtrusionLoopRole,
    tree::transform_nodes,
};
use crate::geometry::{Point, Polygon};

#[test]
fn task22o8_maps_all_loop_roles_and_chains_only_overhang_paths() {
    let roles = [
        (PendingLoopRole::Internal, ExtrusionLoopRole::Internal),
        (PendingLoopRole::Default, ExtrusionLoopRole::Default),
        (PendingLoopRole::Hole, ExtrusionLoopRole::Hole),
    ];
    for (pending, expected) in roles {
        let raw_paths = mixed_paths();
        let point_buffers = raw_paths
            .iter()
            .map(|path| path.polyline.points.as_ptr())
            .collect::<Vec<_>>();
        let raw = RawPathNode {
            paths: raw_paths,
            children: Vec::new(),
        };
        let nodes = transform_nodes(vec![raw], &[seed(pending, Vec::new())], overhang_branch());
        let extrusion_loop = nodes[0].extrusion_loop.as_ref().unwrap();
        assert_eq!(extrusion_loop.role, expected);
        assert_eq!(xyz(&extrusion_loop.paths[0]), vec![(0, 0, 1), (2, 0, 2)]);
        assert_eq!(
            xyz(&extrusion_loop.paths[1]),
            vec![(4, 0, 5), (6, 0, 4), (8, 0, 3)]
        );
        assert_eq!(
            (
                extrusion_loop.paths[1].mm3_per_mm,
                extrusion_loop.paths[1].width,
                extrusion_loop.paths[1].height
            ),
            (2.0, 0.42, 0.22)
        );
        let moved_buffers = extrusion_loop
            .paths
            .iter()
            .map(|path| path.polyline.points.as_ptr())
            .collect::<Vec<_>>();
        assert!(
            point_buffers
                .iter()
                .all(|buffer| moved_buffers.contains(buffer))
        );
    }
}

#[test]
fn task22o8_empty_overhang_is_none_but_ordinary_bypasses_continue_and_chaining() {
    let child_seed = seed(PendingLoopRole::Hole, Vec::new());
    let empty_seed = seed(PendingLoopRole::Default, vec![child_seed]);
    let empty_raw = RawPathNode {
        paths: Vec::new(),
        children: vec![RawPathNode {
            paths: vec![path(&[(1, 1, 0), (2, 1, 0)], 4.0, 0.4, 0.2)],
            children: Vec::new(),
        }],
    };
    let nodes = transform_nodes(vec![empty_raw], &[empty_seed], overhang_branch());
    assert!(nodes[0].extrusion_loop.is_none());
    assert_eq!(nodes[0].children.len(), 1);
    assert!(nodes[0].children[0].extrusion_loop.is_some());

    let original = mixed_paths();
    let expected: Vec<_> = original.iter().map(xyz).collect();
    let ordinary = transform_nodes(
        vec![RawPathNode {
            paths: original,
            children: Vec::new(),
        }],
        &[seed(PendingLoopRole::Internal, Vec::new())],
        PendingPathBranch::OrdinaryUnsplit {
            detect_overhang_wall: false,
            layer_id: 5,
            raft_layers: 0,
        },
    );
    let actual: Vec<_> = ordinary[0]
        .extrusion_loop
        .as_ref()
        .unwrap()
        .paths
        .iter()
        .map(xyz)
        .collect();
    assert_eq!(actual, expected);
    let paths = &ordinary[0].extrusion_loop.as_ref().unwrap().paths;
    assert_eq!(paths[0].role, ExtrusionRole::Perimeter);
    assert_eq!(
        (paths[0].mm3_per_mm, paths[0].width, paths[0].height),
        (1.0, 0.41, 0.21)
    );
    assert_eq!(
        (paths[1].mm3_per_mm, paths[1].width, paths[1].height),
        (2.0, 0.42, 0.22)
    );
}

fn overhang_branch() -> PendingPathBranch {
    PendingPathBranch::OverhangClipping {
        detect_overhang_wall: true,
        layer_id: 5,
        raft_layers: 0,
    }
}

fn seed(role: PendingLoopRole, children: Vec<TraversalSeed>) -> TraversalSeed {
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
        loop_role: role,
        route: LowerFlowRoute::Internal,
        width: 0.4,
        mm3_per_mm: 1.0,
        children,
    }
}

fn mixed_paths() -> Vec<ExtrusionPath> {
    vec![
        path(&[(0, 0, 1), (2, 0, 2)], 1.0, 0.41, 0.21),
        path(&[(8, 0, 3), (6, 0, 4), (4, 0, 5)], 2.0, 0.42, 0.22),
    ]
}

fn path(points: &[(i64, i64, i64)], mm3: f64, width: f32, height: f32) -> ExtrusionPath {
    ExtrusionPath {
        polyline: Polyline3 {
            points: points.iter().map(|&(x, y, z)| Point3 { x, y, z }).collect(),
            fitting: Vec::new(),
        },
        role: ExtrusionRole::Perimeter,
        mm3_per_mm: mm3,
        width,
        height,
    }
}

fn xyz(path: &ExtrusionPath) -> Vec<(i64, i64, i64)> {
    path.polyline
        .points
        .iter()
        .map(|p| (p.x, p.y, p.z))
        .collect()
}
