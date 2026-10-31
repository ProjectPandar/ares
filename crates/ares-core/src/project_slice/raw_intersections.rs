use std::{borrow::Cow, num::NonZeroU32, ops::Deref};

use crate::{
    Point3d, ProjectObject, ProjectVolumeType, SliceError,
    geometry::CoordinateScale,
    mesh_slicer::{IntersectionLine, RawIntersectionBudget, slice_mesh_on_planes},
    options::FilamentRegionSourceOptions,
    project::effective_config::types::ResolvedProjectObject,
};

use super::layers::PlannedPrintObject;

#[path = "raw_intersections/ranges.rs"]
mod ranges;

const DENSE_SLOT_LIMIT: usize = 1_000_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct VolumeOrdinal(NonZeroU32);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ProjectedVolume {
    source_volume_index: usize,
    volume_ordinal: VolumeOrdinal,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ProjectedPrintObject {
    plan: PlannedPrintObject,
    volumes: Vec<ProjectedVolume>,
}

impl ProjectedPrintObject {
    pub(super) fn into_parts(self) -> (PlannedPrintObject, Vec<ProjectedVolume>) {
        (self.plan, self.volumes)
    }
}

impl Deref for ProjectedPrintObject {
    type Target = PlannedPrintObject;

    fn deref(&self) -> &Self::Target {
        &self.plan
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct RawVolumeIntersections {
    source_volume_index: usize,
    volume_ordinal: VolumeOrdinal,
    volume_type: ProjectVolumeType,
    layers: Vec<Vec<IntersectionLine>>,
}

impl RawVolumeIntersections {
    pub(super) fn into_parts(self) -> (usize, u32, ProjectVolumeType, Vec<Vec<IntersectionLine>>) {
        (
            self.source_volume_index,
            self.volume_ordinal.0.get(),
            self.volume_type,
            self.layers,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct IntersectedPrintObject {
    pub(super) plan: PlannedPrintObject,
    volumes: Vec<RawVolumeIntersections>,
}

impl IntersectedPrintObject {
    pub(super) fn into_parts(self) -> (PlannedPrintObject, Vec<RawVolumeIntersections>) {
        (self.plan, self.volumes)
    }
}

impl Deref for IntersectedPrintObject {
    type Target = PlannedPrintObject;

    fn deref(&self) -> &Self::Target {
        &self.plan
    }
}

pub(super) fn prepare_projected_objects(
    source_objects: &[ProjectObject],
    resolved_objects: &[ResolvedProjectObject],
    planned_objects: Vec<PlannedPrintObject>,
) -> Result<Vec<ProjectedPrintObject>, SliceError> {
    validate_layer_ranges(source_objects, resolved_objects)?;

    let projected_objects = planned_objects
        .into_iter()
        .map(|plan| {
            let source = &source_objects[plan.source_object_index];
            let volumes = project_volumes(source);
            ProjectedPrintObject { plan, volumes }
        })
        .collect::<Vec<_>>();
    let counts = projected_objects
        .iter()
        .map(|object| (object.plan.layers.len(), object.volumes.len()))
        .collect::<Vec<_>>();
    validate_dense_slot_counts(&counts)?;
    Ok(projected_objects)
}

pub(super) fn intersect_projected_objects(
    source_objects: &[ProjectObject],
    resolved_objects: &[ResolvedProjectObject],
    projected_objects: Vec<ProjectedPrintObject>,
    scale: CoordinateScale,
    (filament, logical_count): (&FilamentRegionSourceOptions, usize),
) -> Result<Vec<IntersectedPrintObject>, SliceError> {
    let mut budget = RawIntersectionBudget::new();
    let mut intersected_objects = Vec::with_capacity(projected_objects.len());
    for projected_object in projected_objects {
        let (plan, projected_volumes) = projected_object.into_parts();
        let source = &source_objects[plan.source_object_index];
        let resolved = resolved_objects
            .iter()
            .find(|resolved| resolved.source_object_index == plan.source_object_index)
            .expect("planned object must have resolved configuration");
        let active_ranges = (resolved.layer_candidates.len() > 1)
            .then(|| ranges::active_volume_ranges(source, resolved, filament, logical_count));
        let center =
            super::centering::center(source, &resolved.print_objects[plan.transform_index], scale)?;
        let centered_object_transform = resolved.print_objects[plan.transform_index]
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
        let mut volumes = Vec::with_capacity(projected_volumes.len());
        for projected_volume in projected_volumes {
            let source_volume = &source.volumes()[projected_volume.source_volume_index];
            let slicing_transform = centered_object_transform
                .then(source_volume.transform())
                .prescaled_xy(scale.factor());
            let vertices =
                transformed_vertices(source_volume.mesh().vertices(), slicing_transform)?;
            // PrintObjectSlice.cpp:84-92: a left-handed slicing trafo
            // (negative linear determinant) flips the triangle winding
            // before slicing — `its_flip_triangles` swaps the last two
            // indices (TriangleMesh.cpp:736-740). `prescaled_xy` applies a
            // positive-uniform scale, so the sign test is invariant to it.
            let triangles = if slicing_transform.linear_determinant() < 0.0 {
                Cow::Owned(
                    source_volume
                        .mesh()
                        .triangles()
                        .iter()
                        .map(|&[a, b, c]| [a, c, b])
                        .collect::<Vec<_>>(),
                )
            } else {
                Cow::Borrowed(source_volume.mesh().triangles())
            };
            let layers = match &active_ranges {
                Some(active) => ranges::slice_volume(
                    &vertices,
                    &triangles,
                    &planes,
                    &active[projected_volume.source_volume_index],
                    &mut budget,
                )?,
                None => slice_mesh_on_planes(&vertices, &triangles, &planes, &mut budget)?,
            };
            // PrintObjectSlice.cpp:221-225 drops volumes with no filtered layers.
            if active_ranges.is_some() && layers.is_empty() {
                continue;
            }
            volumes.push(RawVolumeIntersections {
                source_volume_index: projected_volume.source_volume_index,
                volume_ordinal: projected_volume.volume_ordinal,
                volume_type: source_volume.volume_type(),
                layers,
            });
        }
        intersected_objects.push(IntersectedPrintObject { plan, volumes });
    }
    Ok(intersected_objects)
}

pub(super) fn raw_center_unscaled(source: &ProjectObject) -> Result<[f64; 2], SliceError> {
    let object_transform = source
        .instances()
        .first()
        .expect("resolved project object must have a source instance")
        .transform()
        .without_translation();
    let mut bounds: Option<[f64; 4]> = None;
    for volume in source.volumes().iter().filter(|volume| {
        volume.volume_type() == ProjectVolumeType::ModelPart
            && !volume.mesh().triangles().is_empty()
    }) {
        let transform = object_transform.then(volume.transform());
        for vertex in volume.mesh().vertices() {
            let transformed = transform.transform_point(*vertex);
            let finite = [transformed.x, transformed.y, transformed.z]
                .into_iter()
                .all(f64::is_finite);
            if !finite {
                return Err(coordinate_error());
            }
            bounds = Some(match bounds {
                Some([min_x, min_y, max_x, max_y]) => [
                    min_x.min(transformed.x),
                    min_y.min(transformed.y),
                    max_x.max(transformed.x),
                    max_y.max(transformed.y),
                ],
                None => [transformed.x, transformed.y, transformed.x, transformed.y],
            });
        }
    }
    let [min_x, min_y, max_x, max_y] =
        bounds.expect("resolved project object must have a nonempty model part");
    let center_x = (min_x + max_x) * 0.5;
    let center_y = (min_y + max_y) * 0.5;
    Ok([center_x, center_y])
}

pub(super) fn transformed_vertices(
    vertices: &[Point3d],
    transform: crate::Transform3d,
) -> Result<Vec<[f32; 3]>, SliceError> {
    vertices
        .iter()
        .map(|vertex| {
            let transformed = transform.transform_point_f32(*vertex);
            let xy_is_valid = transformed[..2]
                .iter()
                .copied()
                .all(scaled_coordinate_is_valid);
            if xy_is_valid && transformed[2].is_finite() {
                Ok(transformed)
            } else {
                Err(coordinate_error())
            }
        })
        .collect()
}

fn scaled_coordinate_is_valid(value: f32) -> bool {
    (i64::MIN as f64..-(i64::MIN as f64)).contains(&f64::from(value))
}

pub(super) fn coordinate_error() -> SliceError {
    SliceError::InvalidInput(
        "project mesh slicing coordinate is nonfinite or outside the scaled coordinate range"
            .to_owned(),
    )
}

// Profile selection has already rejected regeneration with a missing height;
// here retain the existing no-profile rejection for the raw preflight boundary.
fn validate_layer_ranges(
    source_objects: &[ProjectObject],
    resolved_objects: &[ResolvedProjectObject],
) -> Result<(), SliceError> {
    if resolved_objects.iter().any(|resolved| {
        source_objects[resolved.source_object_index]
            .layer_config_ranges()
            .iter()
            .any(|range| range.layer_height().is_none())
            && source_objects[resolved.source_object_index]
                .layer_height_profile()
                .is_empty()
    }) {
        return unsupported("layer_config_ranges");
    }
    Ok(())
}

fn project_volumes(source: &ProjectObject) -> Vec<ProjectedVolume> {
    source
        .volumes()
        .iter()
        .enumerate()
        .filter(|(_, volume)| !volume.mesh().triangles().is_empty())
        .enumerate()
        .filter_map(|(ordinal_index, (source_volume_index, volume))| {
            let ordinal = u32::try_from(ordinal_index)
                .ok()
                .and_then(|index| index.checked_add(1))
                .and_then(NonZeroU32::new)
                .expect("expanded model budget bounds nonempty volume ordinals");
            matches!(
                volume.volume_type(),
                ProjectVolumeType::ModelPart
                    | ProjectVolumeType::NegativeVolume
                    | ProjectVolumeType::ParameterModifier
            )
            .then_some(ProjectedVolume {
                source_volume_index,
                volume_ordinal: VolumeOrdinal(ordinal),
            })
        })
        .collect()
}

pub(super) fn validate_dense_slot_counts(counts: &[(usize, usize)]) -> Result<usize, SliceError> {
    counts.iter().try_fold(0_usize, |used, &(layers, volumes)| {
        layers
            .checked_mul(volumes)
            .and_then(|slots| used.checked_add(slots))
            .filter(|total| *total <= DENSE_SLOT_LIMIT)
            .ok_or_else(dense_slot_limit_error)
    })
}

fn dense_slot_limit_error() -> SliceError {
    SliceError::InvalidInput(
        "project raw intersection layer slot count exceeds supported limit of 1000000".to_owned(),
    )
}

fn unsupported<T>(feature: &str) -> Result<T, SliceError> {
    Err(SliceError::UnsupportedProjectFeature(feature.to_owned()))
}
