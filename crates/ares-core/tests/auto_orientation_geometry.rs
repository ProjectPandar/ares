//! Quickhull failure modes: reversed or missing facets, interior/duplicate vertices changing
//! the boundary, coplanar triangulation changing area, nondeterministic ordering, and
//! non-finite or rank-deficient input. Native whole-output coverage lives in ares-cli.
use ares_core::convex_hull_3d;

#[test]
fn tetrahedron_has_four_outward_facets() {
    let points = [[0., 0., 0.], [11., 0., 2.], [1., 7., 3.], [2., 1., 17.]];
    let hull = convex_hull_3d(&points).unwrap();
    assert_eq!(hull.facets.len(), 4);
    assert_eq!(hull.normals.len(), 4);
    assert_outward(&hull.vertices, &hull.facets, &hull.normals);
    let repeated = convex_hull_3d(&points).unwrap();
    assert_eq!(hull, repeated);
}

#[test]
fn coplanar_box_facets_keep_surface_area_with_duplicates_and_interior_points() {
    let mut points = Vec::new();
    for x in [0., 3.] {
        for y in [0., 5.] {
            for z in [0., 7.] {
                points.push([x, y, z]);
            }
        }
    }
    points.extend([[0., 0., 0.], [1., 2., 3.], [1., 0., 2.], [3., 2., 4.]]);
    let hull = convex_hull_3d(&points).unwrap();
    assert_outward(&hull.vertices, &hull.facets, &hull.normals);
    let area: f32 = hull
        .facets
        .iter()
        .map(|indices| {
            let [a, b, c] = indices.map(|index| hull.vertices[index as usize]);
            let u = sub(b, a);
            let v = sub(c, a);
            let cross = cross(u, v);
            dot(cross, cross).sqrt() * 0.5
        })
        .sum();
    assert!((area - 142.).abs() < 1e-4, "area {area}");
    assert_eq!(hull, convex_hull_3d(&points).unwrap());
}

#[test]
fn degenerate_and_nonfinite_inputs_have_no_three_dimensional_hull() {
    for points in [
        Vec::new(),
        vec![[0., 0., 0.]; 4],
        vec![[0., 0., 0.], [1., 0., 0.], [2., 0., 0.], [3., 0., 0.]],
        vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.], [1., 1., 0.]],
        vec![[f32::NAN, 0., 0.], [1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
    ] {
        assert!(convex_hull_3d(&points).is_none());
    }
}

fn assert_outward(vertices: &[[f32; 3]], facets: &[[u32; 3]], normals: &[[f32; 3]]) {
    for (indices, normal) in facets.iter().zip(normals) {
        let [a, b, c] = indices.map(|index| vertices[index as usize]);
        let area_normal = cross(sub(b, a), sub(c, a));
        assert!(dot(area_normal, *normal) > 0.);
        assert!((dot(*normal, *normal) - 1.).abs() < 1e-5);
        for point in vertices {
            assert!(dot(sub(*point, a), *normal) <= 1e-4);
        }
        for [u, v] in [
            [indices[0], indices[1]],
            [indices[1], indices[2]],
            [indices[2], indices[0]],
        ] {
            let reversed = facets
                .iter()
                .filter(|face| {
                    [[face[0], face[1]], [face[1], face[2]], [face[2], face[0]]].contains(&[v, u])
                })
                .count();
            assert_eq!(reversed, 1, "non-manifold hull edge {u} -> {v}");
        }
    }
}
fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
