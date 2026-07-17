use std::{collections::BTreeSet, num::NonZeroU32, ops::Deref};

use crate::{
    Point3d, ProjectObject, ProjectVolumeType, SliceError,
    geometry::{CoordinateScale, Point},
    mesh_slicer::{IntersectionLine, RawIntersectionBudget, slice_mesh_on_planes},
    project::effective_config::types::ResolvedProjectObject,
};

use super::layers::PlannedPrintObject;

const DENSE_SLOT_LIMIT: usize = 1_000_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct VolumeOrdinal(NonZeroU32);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ProjectedVolume {
    source_volume_index: usize,
    volume_ordinal: VolumeOrdinal,
}

#[cfg(test)]
impl ProjectedVolume {
    pub(super) fn source_volume_index(&self) -> usize {
        self.source_volume_index
    }
    pub(super) fn ordinal(&self) -> u32 {
        self.volume_ordinal.0.get()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ProjectedPrintObject {
    plan: PlannedPrintObject,
    volumes: Vec<ProjectedVolume>,
}

impl ProjectedPrintObject {
    #[cfg(test)]
    pub(super) fn volumes(&self) -> &[ProjectedVolume] {
        &self.volumes
    }
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
    volume_ordinal: VolumeOrdinal,
    volume_type: ProjectVolumeType,
    layers: Vec<Vec<IntersectionLine>>,
}

impl RawVolumeIntersections {
    #[cfg(test)]
    pub(super) fn ordinal(&self) -> u32 {
        self.volume_ordinal.0.get()
    }
    #[cfg(test)]
    pub(super) const fn volume_type(&self) -> ProjectVolumeType {
        self.volume_type
    }
    #[cfg(test)]
    pub(super) fn layers(&self) -> &[Vec<IntersectionLine>] {
        &self.layers
    }
    pub(super) fn into_parts(self) -> (u32, ProjectVolumeType, Vec<Vec<IntersectionLine>>) {
        (self.volume_ordinal.0.get(), self.volume_type, self.layers)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct IntersectedPrintObject {
    pub(super) plan: PlannedPrintObject,
    volumes: Vec<RawVolumeIntersections>,
}

impl IntersectedPrintObject {
    #[cfg(test)]
    pub(super) fn volumes(&self) -> &[RawVolumeIntersections] {
        &self.volumes
    }
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
    validate_centering(source_objects, resolved_objects)?;
    validate_shared_meshes(source_objects)?;

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
        let center = raw_center(source, scale)?;
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
            let layers = slice_mesh_on_planes(
                &vertices,
                source_volume.mesh().triangles(),
                &planes,
                &mut budget,
            )?;
            volumes.push(RawVolumeIntersections {
                volume_ordinal: projected_volume.volume_ordinal,
                volume_type: source_volume.volume_type(),
                layers,
            });
        }
        intersected_objects.push(IntersectedPrintObject { plan, volumes });
    }
    Ok(intersected_objects)
}

pub(super) fn raw_center(
    source: &ProjectObject,
    scale: CoordinateScale,
) -> Result<Point, SliceError> {
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
    let x = scale.checked_scale(center_x).ok_or_else(coordinate_error)?;
    let y = scale.checked_scale(center_y).ok_or_else(coordinate_error)?;
    Ok(Point::new(x, y))
}

fn transformed_vertices(
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

fn coordinate_error() -> SliceError {
    SliceError::InvalidInput(
        "project mesh slicing coordinate is nonfinite or outside the scaled coordinate range"
            .to_owned(),
    )
}

fn validate_layer_ranges(
    source_objects: &[ProjectObject],
    resolved_objects: &[ResolvedProjectObject],
) -> Result<(), SliceError> {
    if resolved_objects.iter().any(|resolved| {
        !source_objects[resolved.source_object_index]
            .layer_config_ranges()
            .is_empty()
    }) {
        return unsupported("layer_config_ranges");
    }
    Ok(())
}

fn validate_centering(
    source_objects: &[ProjectObject],
    resolved_objects: &[ResolvedProjectObject],
) -> Result<(), SliceError> {
    for resolved in resolved_objects {
        let [print_object] = resolved.print_objects.as_slice() else {
            return unsupported("print_object_centering");
        };
        let source = &source_objects[resolved.source_object_index];
        let first_source_transform = source
            .instances()
            .first()
            .expect("resolved project object must have a source instance")
            .transform()
            .without_xy_translation();
        let same_transform = print_object
            .transform
            .fixed_order_equal(first_source_transform);
        if !same_transform {
            return unsupported("print_object_centering");
        }
    }
    Ok(())
}

fn validate_shared_meshes(source_objects: &[ProjectObject]) -> Result<(), SliceError> {
    let mut source_ids = BTreeSet::new();
    for volume in source_objects
        .iter()
        .flat_map(ProjectObject::volumes)
        .filter(|volume| !volume.mesh().triangles().is_empty())
    {
        if volume.has_mesh_shared() || !source_ids.insert(volume.id()) {
            return unsupported("shared_mesh_centering");
        }
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
