use crate::{
    ProjectObject, ProjectVolumeType, SliceError,
    project::effective_config::types::ResolvedProjectObject,
};

pub(super) fn participating_object_heights(
    source_objects: &[ProjectObject],
    resolved_objects: &[ResolvedProjectObject],
) -> Result<Vec<f64>, SliceError> {
    resolved_objects
        .iter()
        .map(|resolved| object_height(&source_objects[resolved.source_object_index]))
        .collect()
}

pub(super) fn object_height(object: &ProjectObject) -> Result<f64, SliceError> {
    let instance_transform = object.instances()[0].transform();
    let mut bounds: Option<(f64, f64)> = None;

    for volume in object
        .volumes()
        .iter()
        .filter(|volume| volume.volume_type() == ProjectVolumeType::ModelPart)
    {
        let transform = instance_transform.then(volume.transform());
        for &vertex in volume.mesh().vertices() {
            let transformed = transform.transform_point(vertex);
            if !transformed.x.is_finite()
                || !transformed.y.is_finite()
                || !transformed.z.is_finite()
            {
                return invalid_bounds();
            }
            bounds = Some(
                bounds.map_or((transformed.z, transformed.z), |(min_z, max_z)| {
                    (min_z.min(transformed.z), max_z.max(transformed.z))
                }),
            );
        }
    }

    let Some((_min_z, max_z)) = bounds else {
        return invalid_bounds();
    };
    if max_z <= 0.0 {
        return invalid_bounds();
    }
    Ok(max_z)
}

fn invalid_bounds<T>() -> Result<T, SliceError> {
    Err(SliceError::InvalidInput(
        "project-object Z bounds require finite model-part vertices and a positive maximum Z"
            .to_owned(),
    ))
}
