use crate::{
    Point3d, ProjectObject, ProjectVolumeType, SliceError, Transform3d,
    project::effective_config::types::ResolvedProjectObject,
};

pub(super) fn validate(
    has_painted_layer_height_profile: bool,
    source_objects: &[ProjectObject],
    resolved_objects: &[ResolvedProjectObject],
    _spiral_mode: bool,
) -> Result<(), SliceError> {
    if has_painted_layer_height_profile {
        return unsupported("layer_height_profile");
    }
    if resolved_objects.iter().any(|resolved| {
        source_objects[resolved.source_object_index]
            .layer_config_ranges()
            .iter()
            .any(|range| range.layer_height().is_some())
    }) {
        return unsupported("layer_height");
    }
    if resolved_objects
        .iter()
        .any(|resolved| resolved.object.raft_layers.0 != 0)
    {
        return unsupported("raft_layers");
    }
    if resolved_objects.iter().any(|resolved| {
        zaa_requested(source_objects, resolved)
            && !zaa_is_provably_inactive(&source_objects[resolved.source_object_index], resolved)
    }) {
        return unsupported("zaa_enabled");
    }
    Ok(())
}

fn zaa_requested(source_objects: &[ProjectObject], resolved: &ResolvedProjectObject) -> bool {
    resolved
        .layer_candidates
        .iter()
        .flat_map(|candidate| &candidate.model_parts)
        .any(|part| part.region.zaa_enabled.0)
        || source_objects[resolved.source_object_index]
            .volumes()
            .iter()
            .filter(|volume| volume.volume_type() == ProjectVolumeType::ParameterModifier)
            .any(|volume| {
                volume
                    .region_overrides()
                    .zaa_enabled
                    .is_some_and(|value| value.0)
            })
}

fn zaa_is_provably_inactive(source: &ProjectObject, resolved: &ResolvedProjectObject) -> bool {
    if source
        .volumes()
        .iter()
        .filter(|volume| volume.volume_type() == ProjectVolumeType::ParameterModifier)
        .any(|volume| {
            volume
                .region_overrides()
                .zaa_enabled
                .is_some_and(|value| value.0)
        })
    {
        return false;
    }
    let model_parts = source
        .volumes()
        .iter()
        .filter(|volume| volume.volume_type() == ProjectVolumeType::ModelPart)
        .collect::<Vec<_>>();
    !model_parts.is_empty()
        && model_parts.into_iter().all(|volume| {
            resolved.print_objects.iter().all(|print_object| {
                let transform = print_object.transform.then(volume.transform());
                is_axis_aligned_box(volume.mesh(), transform)
            })
        })
}

fn is_axis_aligned_box(mesh: &crate::ProjectMesh, transform: Transform3d) -> bool {
    if mesh.vertices().is_empty() || mesh.triangles().is_empty() {
        return false;
    }
    let vertices = mesh
        .vertices()
        .iter()
        .copied()
        .map(|point| transform.transform_point(point))
        .collect::<Vec<_>>();
    let bounds = [
        extrema(&vertices, |point| point.x),
        extrema(&vertices, |point| point.y),
        extrema(&vertices, |point| point.z),
    ];
    bounds.iter().all(|(minimum, maximum)| maximum > minimum)
        && vertices.iter().all(|point| {
            [point.x, point.y, point.z]
                .into_iter()
                .zip(bounds)
                .all(|(value, (minimum, maximum))| close(value, minimum) || close(value, maximum))
        })
        && mesh.triangles().iter().all(|triangle| {
            let points = triangle.map(|index| vertices[index as usize]);
            let normal = cross(
                subtract(points[1], points[0]),
                subtract(points[2], points[0]),
            );
            (close(normal.x, 0.0) && close(normal.y, 0.0)) || close(normal.z, 0.0)
        })
}

fn extrema(vertices: &[Point3d], coordinate: impl Fn(&Point3d) -> f64) -> (f64, f64) {
    vertices.iter().map(coordinate).fold(
        (f64::INFINITY, f64::NEG_INFINITY),
        |(minimum, maximum), value| (minimum.min(value), maximum.max(value)),
    )
}

fn subtract(first: Point3d, second: Point3d) -> Point3d {
    Point3d::new(first.x - second.x, first.y - second.y, first.z - second.z)
}

fn cross(first: Point3d, second: Point3d) -> Point3d {
    Point3d::new(
        first.y * second.z - first.z * second.y,
        first.z * second.x - first.x * second.z,
        first.x * second.y - first.y * second.x,
    )
}

fn close(first: f64, second: f64) -> bool {
    (first - second).abs() <= 1.0e-9
}

fn unsupported<T>(key: &str) -> Result<T, SliceError> {
    Err(SliceError::UnsupportedProjectFeature(key.to_owned()))
}
