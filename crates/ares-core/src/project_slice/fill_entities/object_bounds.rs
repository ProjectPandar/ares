//! PrintObject::bounding_box and Fill::extended_object_bounding_box.
use crate::{
    Point3d, ProjectObject, ProjectVolumeType, Transform3d,
    geometry::{BoundingBox, CoordinateScale, Point},
};

pub(in crate::project_slice) fn object(
    object: &ProjectObject,
    transform: Transform3d,
    scale: CoordinateScale,
) -> BoundingBox {
    // Model.cpp:1586-1599 / TriangleMesh.cpp:440-446: model parts only, float
    // mesh vertices cast to double before the no-offset instance transform.
    let transform = transform.without_translation();
    let mut min = [f64::INFINITY; 2];
    let mut max = [f64::NEG_INFINITY; 2];
    for volume in object.volumes() {
        if volume.volume_type() != ProjectVolumeType::ModelPart {
            continue;
        }
        let matrix = transform.then(volume.transform());
        for vertex in volume.mesh().vertices() {
            let point = matrix.transform_point(Point3d::new(
                f64::from(vertex.x as f32),
                f64::from(vertex.y as f32),
                f64::from(vertex.z as f32),
            ));
            for (axis, value) in [point.x, point.y].into_iter().enumerate() {
                min[axis] = min[axis].min(value);
                max[axis] = max[axis].max(value);
            }
        }
    }
    // PrintObject.cpp:112 truncates m_size before Print.hpp:347 halves it.
    let half_x = ((max[0] - min[0]) / scale.factor()) as i64 / 2;
    let half_y = ((max[1] - min[1]) / scale.factor()) as i64 / 2;
    BoundingBox::new(Point::new(-half_x, -half_y), Point::new(half_x, half_y))
}

pub(in crate::project_slice) fn extended(
    object: &ProjectObject,
    transform: Transform3d,
    scale: CoordinateScale,
) -> BoundingBox {
    let bounds = self::object(object, transform, scale);
    // FillBase.cpp:1569-1578 transpose-merges the centered box, then scales
    // both corners about the origin. The resulting XY extent is square.
    let half = bounds.max().x().max(bounds.max().y());
    let extent = (half as f64 * std::f64::consts::SQRT_2) as i64;
    BoundingBox::new(Point::new(-extent, -extent), Point::new(extent, extent))
}
