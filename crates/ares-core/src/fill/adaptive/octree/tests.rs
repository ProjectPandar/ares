use super::*;

#[test]
fn cubes_properties_ladder_doubles_from_line_spacing() {
    let props = make_cubes_properties(10.0, 0.4);
    assert!(props.len() >= 2);
    assert!((props[0].edge_length - 0.8).abs() < 1e-12);
    for pair in props.windows(2) {
        assert!((pair[1].edge_length - 2.0 * pair[0].edge_length).abs() < 1e-12);
    }
    assert!(props.last().unwrap().edge_length > 10.0);
}

#[test]
fn cubes_properties_minimum_two_levels() {
    let props = make_cubes_properties(0.5, 0.4);
    assert_eq!(props.len(), 2);
    assert!((props[1].edge_length - 1.6).abs() < 1e-12);
    assert!((props[1].height - 1.6 * 3.0f64.sqrt()).abs() < 1e-12);
}

#[test]
fn octree_root_survives_without_triangles() {
    let vertices = [[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let octree = build_octree(&vertices, &[], &[], 0.4, false, [0.0; 3]);
    assert_eq!(octree.cubes_properties.len(), 2);
    assert!(octree.root_cube.children.iter().all(|c| c.is_none()));
    assert_eq!(octree.cubes_properties.last().unwrap().edge_length, 1.6);
}

#[test]
fn octree_inserts_triangle_through_levels() {
    let vertices = [
        [0.0; 3],
        [10.0, 0.0, 0.0],
        [0.0, 10.0, 0.0],
        [0.0, 0.0, 10.0],
    ];
    let triangles = [[0u32, 1, 2], [0, 2, 3], [0, 1, 3]];
    let octree = build_octree(&vertices, &triangles, &[], 0.4, false, [0.0; 3]);
    let children = octree
        .root_cube
        .children
        .iter()
        .filter(|c| c.is_some())
        .count();
    assert!(children > 0, "triangle must land in at least one child");
}

#[test]
fn sat_rejects_disjoint_triangle() {
    let tri = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
    let far_box = ([5.0, 5.0, 5.0], [6.0, 6.0, 6.0]);
    assert!(!triangle_aabb_intersects(tri[0], tri[1], tri[2], far_box));
    let touching_box = ([0.5, 0.5, -0.5], [2.0, 2.0, 0.5]);
    assert!(triangle_aabb_intersects(
        tri[0],
        tri[1],
        tri[2],
        touching_box
    ));
}

#[test]
fn world_rotation_is_orthonormal() {
    let r = octree_rotation_to_world();
    for row in 0..3 {
        let mut norm2 = 0.0;
        for col in 0..3 {
            norm2 += r[row][col] * r[row][col];
        }
        assert!((norm2 - 1.0).abs() < 1e-12);
    }
}
