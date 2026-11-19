use serde::Serialize;

use super::{
    Bounds, Point3d, ProjectObject, ProjectVolumeType, Transform3d, point_array, volume_bounds,
};
use crate::SliceError;

#[derive(Debug, Serialize)]
pub struct MeshObject<'a> {
    pub bbox_object: Bounds,
    pub instance_count: usize,
    pub name: &'a str,
    pub triangle_count: usize,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub first_instance: Option<MeshInstance>,
}

#[derive(Debug, Serialize)]
pub struct MeshInstance {
    pub bbox_world: Bounds,
    pub instance_offset: [f64; 3],
    pub plane_count: usize,
    pub planes: Vec<MeshPlane>,
}

#[derive(Debug, Serialize)]
pub struct MeshPlane {
    pub area_mm2: f64,
    pub center: [f64; 3],
    pub normal: [f64; 3],
}

impl ProjectObject {
    /// Object-space part geometry and the first instance's plate-space geometry.
    pub fn inspect_mesh(&self) -> Result<MeshObject<'_>, SliceError> {
        let parts = || {
            self.volumes()
                .iter()
                .filter(|v| v.volume_type() == ProjectVolumeType::ModelPart)
        };
        let bounds = |instance| {
            Bounds::from_points(parts().flat_map(|volume| {
                let bounds = volume_bounds(volume, instance);
                [bounds.min, bounds.max]
            }))
        };
        let first_instance = if let Some(instance) = self.instances().first() {
            let matrix = instance.transform();
            let mut planes =
                super::super::lay_on_face::inspect_planes(self, matrix.without_translation())?;
            let plane_count = planes.len();
            planes.truncate(8);
            Some(MeshInstance {
                bbox_world: bounds(matrix),
                instance_offset: point_array(matrix.transform_point(Point3d::new(0., 0., 0.))),
                plane_count,
                planes,
            })
        } else {
            None
        };
        Ok(MeshObject {
            bbox_object: bounds(Transform3d::IDENTITY),
            instance_count: self.instances().len(),
            name: self.name(),
            triangle_count: parts().map(|v| v.mesh().triangles().len()).sum(),
            first_instance,
        })
    }
}
