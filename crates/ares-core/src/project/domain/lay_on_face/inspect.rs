use super::{Transform3d, Vec3, cross, dot, hull_faces, instance_planes, object_hull};
use crate::{
    SliceError,
    project::domain::{ProjectObject, inspection::MeshPlane},
};

pub(in super::super) fn inspect_planes(
    object: &ProjectObject,
    linear: Transform3d,
) -> Result<Vec<MeshPlane>, SliceError> {
    let Some(hull) = object_hull(object)? else {
        return Ok(Vec::new());
    };
    Ok(instance_planes(&hull_faces(&hull), &hull, linear)
        .into_iter()
        .map(|plane| {
            let center = plane.outline.iter().fold([0.0; 3], |sum, point| {
                std::array::from_fn(|i| sum[i] + point[i])
            });
            let center = center.map(|x| x / plane.outline.len() as f64);
            MeshPlane {
                area_mm2: (f64::from(plane.area) * 1000.0).round() / 1000.0,
                center: inverse_transform(linear, inverse_transform(plane.frame, center)),
                normal: plane.normal,
            }
        })
        .collect())
}

fn inverse_transform(matrix: Transform3d, point: Vec3) -> Vec3 {
    let columns: [Vec3; 3] =
        std::array::from_fn(|c| std::array::from_fn(|r| matrix.as_matrix()[r][c]));
    let det_inverse = 1.0 / matrix.linear_determinant();
    std::array::from_fn(|i| {
        let cofactors = cross(columns[(i + 1) % 3], columns[(i + 2) % 3]);
        dot(cofactors.map(|x| x * det_inverse), point)
    })
}
