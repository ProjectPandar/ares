use std::num::{NonZeroU32, NonZeroUsize};

use crate::{
    ProjectObject, ProjectVolumeType,
    project::effective_config::types::ResolvedProjectObject,
    project::region_graph::bounds::{BoundingBox3f, transformed_mesh_bbox},
};

use super::{
    closing::{PostClosingLayer, PostClosingPrintObject},
    layers::PlannedPrintObject,
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct VolumeOccurrenceId(NonZeroU32);

impl VolumeOccurrenceId {
    fn promote(released_ordinal: u32) -> Self {
        Self(NonZeroU32::new(released_ordinal).expect("released volume ordinal must be nonzero"))
    }

    pub(super) const fn get(self) -> u32 {
        self.0.get()
    }
}

pub(super) struct PostBoundsVolume {
    source_volume_index: usize,
    occurrence_id: VolumeOccurrenceId,
    volume_type: ProjectVolumeType,
    layers: Vec<PostClosingLayer>,
}

impl PostBoundsVolume {
    pub(super) const fn source_volume_index(&self) -> usize {
        self.source_volume_index
    }

    pub(super) const fn occurrence_id(&self) -> VolumeOccurrenceId {
        self.occurrence_id
    }

    pub(super) const fn volume_type(&self) -> ProjectVolumeType {
        self.volume_type
    }

    pub(super) fn into_parts(
        self,
    ) -> (
        usize,
        VolumeOccurrenceId,
        ProjectVolumeType,
        Vec<PostClosingLayer>,
    ) {
        (
            self.source_volume_index,
            self.occurrence_id,
            self.volume_type,
            self.layers,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct VolumeBound {
    source_volume_index: usize,
    occurrence_id: VolumeOccurrenceId,
    bbox: BoundingBox3f,
}

impl VolumeBound {
    pub(super) const fn source_volume_index(self) -> usize {
        self.source_volume_index
    }

    pub(super) const fn occurrence_id(self) -> VolumeOccurrenceId {
        self.occurrence_id
    }

    pub(super) const fn bbox(self) -> BoundingBox3f {
        self.bbox
    }

    pub(super) const fn into_parts(self) -> (usize, VolumeOccurrenceId, BoundingBox3f) {
        (self.source_volume_index, self.occurrence_id, self.bbox)
    }
}

pub(super) struct BoundedPrintObject {
    plan: PlannedPrintObject,
    volumes: Vec<PostBoundsVolume>,
    source_volume_positions: Vec<Option<NonZeroUsize>>,
    bounds: Vec<VolumeBound>,
}

impl BoundedPrintObject {
    pub(super) fn volume_position_for_source_index(
        &self,
        source_volume_index: usize,
    ) -> Option<usize> {
        self.source_volume_positions[source_volume_index].map(|position| position.get() - 1)
    }

    pub(super) fn bound_index_for_occurrence(
        &self,
        occurrence_id: VolumeOccurrenceId,
    ) -> Option<usize> {
        self.bounds
            .binary_search_by_key(&occurrence_id, |bound| bound.occurrence_id)
            .ok()
    }

    pub(super) fn bound_index_for_source_index(&self, source_volume_index: usize) -> Option<usize> {
        self.volume_for_source_index(source_volume_index)
            .and_then(|volume| self.bound_index_for_occurrence(volume.occurrence_id))
    }

    pub(super) fn volume_for_source_index(
        &self,
        source_volume_index: usize,
    ) -> Option<&PostBoundsVolume> {
        self.volume_position_for_source_index(source_volume_index)
            .map(|position| &self.volumes[position])
    }

    pub(super) fn bound_for_occurrence(
        &self,
        occurrence_id: VolumeOccurrenceId,
    ) -> Option<&VolumeBound> {
        self.bound_index_for_occurrence(occurrence_id)
            .map(|index| &self.bounds[index])
    }

    pub(super) fn bound_at(&self, index: usize) -> &VolumeBound {
        &self.bounds[index]
    }

    pub(super) fn bound_for_source_index(
        &self,
        source_volume_index: usize,
    ) -> Option<&VolumeBound> {
        self.volume_for_source_index(source_volume_index)
            .and_then(|volume| self.bound_for_occurrence(volume.occurrence_id))
    }

    pub(super) fn into_parts(
        self,
    ) -> (PlannedPrintObject, Vec<PostBoundsVolume>, Vec<VolumeBound>) {
        (self.plan, self.volumes, self.bounds)
    }
}

pub(super) fn build_volume_bounds(
    source: &ProjectObject,
    resolved: &ResolvedProjectObject,
    post_i: PostClosingPrintObject,
) -> BoundedPrintObject {
    build_volume_bounds_with_inflation(
        source,
        resolved,
        post_i,
        (resolved.object.xy_contour_compensation.0 as f32).max(0.0),
    )
}

/// PrintApply.cpp:998–999: painted objects suppress XY bounding-box inflation.
pub(super) fn build_volume_bounds_with_inflation(
    source: &ProjectObject,
    resolved: &ResolvedProjectObject,
    post_i: PostClosingPrintObject,
    xy_inflate: f32,
) -> BoundedPrintObject {
    let (plan, post_i_volumes) = post_i.into_parts();
    let object_transform = resolved.print_objects[0].transform;
    let mut volumes = Vec::with_capacity(post_i_volumes.len());
    let mut source_volume_positions = vec![None; source.volumes().len()];
    let mut bounds = Vec::with_capacity(post_i_volumes.len());

    for volume in post_i_volumes {
        let (source_volume_index, released_ordinal, volume_type, layers) = volume.into_parts();
        let occurrence_id = VolumeOccurrenceId::promote(released_ordinal);
        let source_volume = &source.volumes()[source_volume_index];
        let transform = object_transform
            .then(source_volume.transform())
            .without_xy_translation();
        bounds.push(VolumeBound {
            source_volume_index,
            occurrence_id,
            bbox: transformed_mesh_bbox(source_volume.mesh(), transform, xy_inflate),
        });
        source_volume_positions[source_volume_index] = NonZeroUsize::new(volumes.len() + 1);
        volumes.push(PostBoundsVolume {
            source_volume_index,
            occurrence_id,
            volume_type,
            layers,
        });
    }
    bounds.sort_by_key(|bound| bound.occurrence_id);

    BoundedPrintObject {
        plan,
        volumes,
        source_volume_positions,
        bounds,
    }
}

const _: fn(&ProjectObject, &ResolvedProjectObject, PostClosingPrintObject) -> BoundedPrintObject =
    build_volume_bounds;

const _: () = {
    let _ = VolumeOccurrenceId::get;
    let _ = PostBoundsVolume::source_volume_index;
    let _ = PostBoundsVolume::occurrence_id;
    let _ = PostBoundsVolume::volume_type;
    let _ = PostBoundsVolume::into_parts;
    let _ = BoundingBox3f::min;
    let _ = BoundingBox3f::max;
    let _ = BoundingBox3f::contains_z;
    let _ = BoundingBox3f::intersects_xy;
    let _ = BoundingBox3f::intersects;
    let _ = BoundingBox3f::extend;
    let _ = VolumeBound::source_volume_index;
    let _ = VolumeBound::occurrence_id;
    let _ = VolumeBound::bbox;
    let _ = VolumeBound::into_parts;
    let _ = BoundedPrintObject::volume_position_for_source_index;
    let _ = BoundedPrintObject::volume_for_source_index;
    let _ = BoundedPrintObject::bound_index_for_occurrence;
    let _ = BoundedPrintObject::bound_index_for_source_index;
    let _ = BoundedPrintObject::bound_for_occurrence;
    let _ = BoundedPrintObject::bound_at;
    let _ = BoundedPrintObject::bound_for_source_index;
    let _ = BoundedPrintObject::into_parts;
};
