use super::{
    mesh::IndexedMesh,
    path::{Context, contour},
};
use crate::{
    ExtrusionRole, RegionOptions, geometry::CoordinateScale,
    project_slice::perimeters::classic::materialize::Point3,
};

fn plane(height: f32) -> IndexedMesh {
    IndexedMesh::new(
        vec![[-2., -2., height], [2., -2., height], [0., 2., height]],
        vec![[0, 1, 2]],
        0.,
    )
}

#[test]
fn indexed_mesh_nearest_hit_keeps_float_distance_and_face_normal() {
    let mesh = IndexedMesh::new(
        vec![
            [-2., -2., 2.],
            [2., -2., 2.],
            [0., 2., 2.],
            [-2., -2., 1.],
            [2., -2., 1.],
            [0., 2., 1.],
        ],
        vec![[0, 1, 2], [3, 4, 5]],
        0.,
    );
    let hit = mesh.query([0., 0., 0.]);
    assert_eq!(hit.face, Some(1));
    assert_eq!(hit.distance, 1.);
    assert_eq!(hit.normal, [0., 0., 1.]);
}

#[test]
fn indexed_mesh_accepts_backface_and_rejects_nonpositive_or_missing_hits() {
    let mesh = IndexedMesh::new(
        vec![[-2., -2., 1.], [2., -2., 1.], [0., 2., 1.]],
        vec![[2, 1, 0]],
        0.,
    );
    assert_eq!(mesh.query([0., 0., 0.]).normal, [0., 0., -1.]);
    for origin in [[0., 0., 1.], [0., 0., 2.], [10., 10., 0.]] {
        let hit = mesh.query(origin);
        assert_eq!(hit.face, None);
        assert!(hit.distance.is_infinite());
    }
    assert_eq!(
        IndexedMesh::new(Vec::new(), Vec::new(), 0.)
            .query([0., 0., 0.])
            .face,
        None
    );
}

#[test]
fn indexed_mesh_equal_distance_keeps_left_hit_and_rebuild_is_identical() {
    let vertices = vec![[-2., -2., 1.], [2., -2., 1.], [0., 2., 1.]];
    let mesh = IndexedMesh::new(vertices.clone(), vec![[0, 1, 2], [2, 1, 0]], 0.);
    let again = IndexedMesh::new(vertices, vec![[0, 1, 2], [2, 1, 0]], 0.);
    assert_eq!(mesh.tree, again.tree);
    assert_eq!(mesh.query([0., 0., 0.]).face, Some(0));
    assert_eq!(mesh.query([0., 0., 0.]), again.query([0., 0., 0.]));
}

fn context(region: &RegionOptions) -> Context<'_> {
    Context {
        region,
        slice_z: 0.9,
        print_z: 1.,
        layer_height: 0.2,
        scale: CoordinateScale::Normal,
    }
}
fn points() -> Vec<Point3> {
    vec![
        Point3 {
            x: -500_000,
            y: 0,
            z: 0,
        },
        Point3 { x: 0, y: 0, z: 0 },
        Point3 {
            x: 500_000,
            y: 0,
            z: 0,
        },
    ]
}

#[test]
fn contour_flat_roof_retains_original_path_and_far_or_missing_hits_are_inactive() {
    let region = RegionOptions::from_base(&crate::ProjectSettings::default().process.region);
    let ctx = context(&region);
    for mesh in [
        plane(1.),
        plane(2.),
        IndexedMesh::new(Vec::new(), Vec::new(), 0.),
    ] {
        assert!(contour(&points(), ExtrusionRole::TopSolidInfill, 0.4, &mesh, &ctx).is_none());
    }
    assert!(
        contour(
            &points(),
            ExtrusionRole::InternalInfill,
            0.4,
            &plane(0.95),
            &ctx
        )
        .is_none()
    );
}

#[test]
fn contour_source_height_bounds_and_original_junction_survive() {
    let region = RegionOptions::from_base(&crate::ProjectSettings::default().process.region);
    let ctx = context(&region);
    let top = contour(
        &points(),
        ExtrusionRole::TopSolidInfill,
        0.4,
        &plane(1.07),
        &ctx,
    )
    .unwrap();
    assert!(top.iter().all(|p| p.z == 50_000));
    assert!(top.iter().any(|p| p.x == 0));
    assert!(
        contour(
            &points(),
            ExtrusionRole::ExternalPerimeter,
            0.4,
            &plane(1.04),
            &ctx
        )
        .is_none()
    );
    let lower = contour(
        &points(),
        ExtrusionRole::TopSolidInfill,
        0.4,
        &plane(0.92),
        &ctx,
    )
    .unwrap();
    assert!(lower.iter().all(|p| (p.z + 80_000).abs() <= 1));
}

#[test]
fn contour_ironing_uses_its_distinct_upper_bound() {
    let region = RegionOptions::from_base(&crate::ProjectSettings::default().process.region);
    let ctx = context(&region);
    let ironing = contour(&points(), ExtrusionRole::Ironing, 0.4, &plane(1.15), &ctx).unwrap();
    assert!(ironing.iter().all(|p| (p.z - 150_000).abs() <= 1));
}
