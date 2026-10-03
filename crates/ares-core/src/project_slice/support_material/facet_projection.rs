//! PrintObject.cpp::project_and_append_custom_facets (support painting).

use super::tree::VerticalEnforcer;
use crate::{
    Point3d, ProjectObject, ProjectVolumeType, SliceError,
    geometry::{CoordinateScale, Polygon},
    mesh_slicer::slice_bottom_slabs,
    project::{effective_config::types::ResolvedProjectObject, facets_annotation::strict_facets},
    project_slice::{layers::PlannedPrintObject, raw_intersections::raw_center},
};

pub(super) struct Input<'a> {
    pub(super) source: &'a ProjectObject,
    pub(super) resolved: &'a ResolvedProjectObject,
    pub(super) plan: &'a PlannedPrintObject,
    pub(super) scale: CoordinateScale,
}

pub(super) fn append(
    input: &Input<'_>,
    state: u8,
    target: &mut Vec<Vec<Polygon>>,
) -> Result<Vec<VerticalEnforcer>, SliceError> {
    let Input {
        source,
        resolved,
        plan,
        scale,
    } = *input;
    let mut volumes = source.volumes().iter().filter(|volume| {
        volume.volume_type() == ProjectVolumeType::ModelPart
            && !volume.mesh().supported_facets.mapping.is_empty()
    });
    let Some(first) = volumes.next() else {
        return Ok(Vec::new());
    };
    let center = raw_center(source, scale)?;
    let object_transform = resolved.print_objects[plan.transform_index]
        .transform
        .without_xy_translation()
        .pretranslated(Point3d::new(
            -scale.unscale(center.x()),
            -scale.unscale(center.y()),
            0.0,
        ));
    let planes = plan
        .layers
        .iter()
        .map(|layer| layer.slice_z as f32)
        .collect::<Vec<_>>();
    let mut vertical = Vec::new();
    for volume in std::iter::once(first).chain(volumes) {
        let facets = strict_facets(volume.mesh(), state)?;
        if facets.triangles.is_empty() {
            continue;
        }
        let transform = object_transform.then(volume.transform());
        let scaled_transform = transform.prescaled_xy(scale.factor());
        let vertices = facets
            .vertices
            .into_iter()
            .map(|[x, y, z]| {
                scaled_transform.transform_point_f32(Point3d::new(
                    f64::from(x),
                    f64::from(y),
                    f64::from(z),
                ))
            })
            .collect::<Vec<_>>();
        let projected = slice_bottom_slabs(
            &vertices,
            &facets.triangles,
            &planes,
            transform.linear_determinant() < 0.0,
        )?;
        if target.is_empty() {
            target.resize_with(planes.len(), Vec::new);
        }
        for (layer, polygons) in target.iter_mut().zip(projected.polygons) {
            layer.extend(polygons);
        }
        if state == 1 {
            vertical.extend(
                projected
                    .vertical
                    .into_iter()
                    .map(|(position, normal)| VerticalEnforcer { position, normal }),
            );
        }
    }
    Ok(vertical)
}
