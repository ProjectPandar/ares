//! `PrintObjectSlice.cpp:1545::slice_support_volumes`.

use std::borrow::Cow;

use crate::{
    Point3d, ProjectObject, ProjectVolumeType, SliceError,
    geometry::{CoordinateScale, Polygon, union_polygons_paths},
    mesh_slicer::{
        RawIntersectionBudget, SlicingMode, chain_lines_by_triangle_connectivity, make_loops,
        slice_mesh_on_planes,
    },
    project::effective_config::types::ResolvedProjectObject,
    project_slice::{
        layers::PlannedPrintObject,
        pre_closing_unions::union_layer_polygons,
        raw_intersections::{raw_center, transformed_vertices},
    },
};

pub(super) fn slice_volumes(
    source: &ProjectObject,
    resolved: &ResolvedProjectObject,
    plan: &PlannedPrintObject,
    volume_type: ProjectVolumeType,
    scale: CoordinateScale,
) -> Result<Vec<Vec<Polygon>>, SliceError> {
    let mut volumes = source
        .volumes()
        .iter()
        .filter(|volume| volume.volume_type() == volume_type);
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
    let max_gap = scale.checked_scale(2.0).unwrap();
    let mut budget = RawIntersectionBudget::new();
    let mut slices = vec![Vec::new(); planes.len()];
    let mut merge_layers = vec![false; planes.len()];
    for volume in std::iter::once(first).chain(volumes) {
        let transform = object_transform.then(volume.transform());
        let vertices = transformed_vertices(
            volume.mesh().vertices(),
            transform.prescaled_xy(scale.factor()),
        )?;
        let triangles = if transform.linear_determinant() < 0.0 {
            Cow::Owned(
                volume
                    .mesh()
                    .triangles()
                    .iter()
                    .map(|&[a, b, c]| [a, c, b])
                    .collect::<Vec<_>>(),
            )
        } else {
            Cow::Borrowed(volume.mesh().triangles())
        };
        let layers = slice_mesh_on_planes(&vertices, &triangles, &planes, &mut budget)?;
        for ((target, merge), lines) in slices.iter_mut().zip(&mut merge_layers).zip(layers) {
            let loops = make_loops(chain_lines_by_triangle_connectivity(lines), max_gap);
            let polygons = union_layer_polygons(SlicingMode::Regular, loops.polygons())?;
            if !polygons.is_empty() {
                *merge |= !target.is_empty();
                target.extend(super::super::layer_input::to_polygons(&polygons));
            }
        }
    }
    for (slice, merge) in slices.iter_mut().zip(merge_layers) {
        if merge {
            *slice = union_polygons_paths(slice).map_err(|error| {
                SliceError::InvalidInput(format!("support annotation geometry: {error:?}"))
            })?;
        }
    }
    Ok(slices)
}
