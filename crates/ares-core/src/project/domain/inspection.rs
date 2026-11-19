//! Read-only measurements on loaded model geometry.

mod mesh;
mod paint;
mod statistics;

use serde::Serialize;

use super::{
    Point3d, Project, ProjectInstance, ProjectMesh, ProjectObject, ProjectVolumeType, Transform3d,
};
use crate::{CliProjectTransforms, ConfigOverrides, SliceError};
pub use mesh::{MeshInstance, MeshObject, MeshPlane};
pub use paint::{PaintLayer, PaintLayers, PaintObject, PaintState, PaintSummary, PaintVolume};
pub use statistics::MeshStatistics;

/// Load the same model objects for each inspection action, without preparing a print.
pub fn load(
    bytes: &[u8],
    name: &str,
    overrides: &ConfigOverrides,
    transforms: &CliProjectTransforms,
) -> Result<Project, SliceError> {
    let project = if name.to_ascii_lowercase().ends_with(".stl") {
        let volume = super::multipart::stl_volume(bytes, name, 1)?;
        let bounds = volume_bounds(&volume, Transform3d::IDENTITY);
        let center = Point3d::new(
            100. - (bounds.min[0] + bounds.max[0]) * 0.5,
            100. - (bounds.min[1] + bounds.max[1]) * 0.5,
            -bounds.min[2],
        );
        let instance = ProjectInstance::new(
            [1, 1, 1],
            true,
            true,
            Transform3d::IDENTITY.pretranslated(center),
        );
        let object = ProjectObject::new(
            name.to_owned(),
            1,
            (
                name.to_owned(),
                String::new(),
                Default::default(),
                Default::default(),
            ),
            vec![volume],
            vec![instance],
        );
        Project::from_stl_objects(name, vec![object])
    } else {
        crate::project::load_project_with(bytes, overrides)?
    };
    transforms.apply(project, None)
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Bounds {
    pub max: [f64; 3],
    pub min: [f64; 3],
    pub size: [f64; 3],
}

impl Bounds {
    fn from_points(points: impl IntoIterator<Item = [f64; 3]>) -> Self {
        let mut points = points.into_iter();
        let first = points.next().unwrap_or_default();
        let (mut min, mut max) = (first, first);
        for point in points {
            for axis in 0..3 {
                min[axis] = min[axis].min(point[axis]);
                max[axis] = max[axis].max(point[axis]);
            }
        }
        Self {
            max,
            min,
            size: std::array::from_fn(|i| max[i] - min[i]),
        }
    }
}

fn point_array(p: Point3d) -> [f64; 3] {
    [p.x, p.y, p.z]
}

fn volume_bounds(volume: &super::ProjectVolume, instance: Transform3d) -> Bounds {
    let transform = instance.then(volume.transform());
    Bounds::from_points(
        volume
            .mesh()
            .vertices()
            .iter()
            .map(|&p| point_array(transform.transform_point(p))),
    )
}
