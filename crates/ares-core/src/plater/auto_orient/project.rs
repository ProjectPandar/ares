//! `ModelObject::raw_mesh/mesh` (`Model.cpp:1675-1701`) and the CLI adapter
//! `orient(ModelObject*)` (`Orient.cpp:625-636`).
use super::{Orientation, auto_orient_mesh};
use crate::{Point3d, Project, ProjectMesh, ProjectObject, ProjectVolumeType, SliceError};

/// Scores all transformed model-part instances, not each volume independently.
/// Reflections retain winding (`TriangleMesh.hpp:121`: `fix_left_handed=false`).
/// Native its_merge copies vertices/indices but not face properties
/// (`TriangleMesh.cpp:1448-1458`), so this adapter does not propagate them.
pub fn auto_orient_object(object: &ProjectObject) -> Result<Orientation, SliceError> {
    auto_orient_mesh(&object_mesh(object))
}

/// `orient(ModelInstance*)` (`Orient.cpp:638-648`): score the object mesh,
/// rotate only the selected instance, without centering or a ground drop.
pub fn auto_orient_instance(
    project: &mut Project,
    object_id: u32,
    instance_id: u32,
) -> Result<Orientation, SliceError> {
    let (object, instance) = project
        .objects()
        .iter()
        .enumerate()
        .find_map(|(index, object)| {
            (object.id() == object_id)
                .then(|| {
                    object
                        .instances()
                        .iter()
                        .position(|instance| instance.instance_id() == instance_id)
                        .map(|instance| (index, instance))
                })
                .flatten()
        })
        .ok_or_else(|| {
            SliceError::InvalidInput(format!(
                "project has no instance ({object_id}, {instance_id})"
            ))
        })?;
    let result = auto_orient_object(&project.objects()[object])?;
    let previous = project.objects()[object].instances()[instance].transform();
    let transform =
        super::super::transformation::rotated_instance(previous, result.rotation_matrix);
    project.set_instance_transform(object, instance, transform);
    Ok(result)
}

pub(crate) fn apply_cli_orientation(
    project: &mut Project,
    plate: Option<u32>,
) -> Result<(), SliceError> {
    // An explicit plate's importer membership is established before model actions;
    // do not rematerialize pre-action volume transforms in later plate selection.
    project.prepare_import_scope(plate);
    for index in 0..project.objects().len() {
        let result = auto_orient_object(&project.objects()[index])?;
        let angles = super::super::eigen::angle_axis_euler(result.angle, result.axis);
        project.apply_object_orientation(index, angles);
    }
    Ok(())
}

fn object_mesh(object: &ProjectObject) -> ProjectMesh {
    let mut raw_vertices = Vec::new();
    let mut raw_triangles = Vec::new();
    for volume in object
        .volumes()
        .iter()
        .filter(|v| v.volume_type() == ProjectVolumeType::ModelPart)
    {
        let offset = raw_vertices.len() as u32;
        raw_vertices.extend(volume.mesh().vertices().iter().map(|vertex| {
            let p = volume.transform().transform_point_f32(*vertex);
            Point3d::new(f64::from(p[0]), f64::from(p[1]), f64::from(p[2]))
        }));
        raw_triangles.extend(
            volume
                .mesh()
                .triangles()
                .iter()
                .map(|face| face.map(|index| offset + index)),
        );
    }
    let mut vertices = Vec::with_capacity(raw_vertices.len() * object.instances().len());
    let mut triangles = Vec::with_capacity(raw_triangles.len() * object.instances().len());
    for instance in object.instances() {
        let offset = vertices.len() as u32;
        vertices.extend(raw_vertices.iter().map(|vertex| {
            let p = instance.transform().transform_point_f32(*vertex);
            Point3d::new(f64::from(p[0]), f64::from(p[1]), f64::from(p[2]))
        }));
        triangles.extend(
            raw_triangles
                .iter()
                .map(|face| face.map(|index| offset + index)),
        );
    }
    ProjectMesh::new(vertices, triangles, Vec::new())
}
