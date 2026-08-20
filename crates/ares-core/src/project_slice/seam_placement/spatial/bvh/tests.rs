use super::{TriangleBvh, TriangleMesh, Vec3};
use crate::project_slice::seam_placement::mesh::Triangle;

#[test]
fn first_hit_keeps_nearer_triangle_when_far_bounds_overlap() {
    let mesh = TriangleMesh::new(vec![
        Triangle::new(
            Vec3::new(-2.0, -2.0, 1.0),
            Vec3::new(2.0, -2.0, 1.0),
            Vec3::new(0.0, 2.0, 1.0),
        ),
        Triangle::new(
            Vec3::new(-1.0, -1.0, 0.5),
            Vec3::new(1.0, -1.0, 0.5),
            Vec3::new(0.0, 1.0, 5.0),
        ),
    ]);
    let tree = TriangleBvh::new(&mesh);

    assert_eq!(
        tree.first_hit(&mesh, Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0),),
        Some(0)
    );
}
