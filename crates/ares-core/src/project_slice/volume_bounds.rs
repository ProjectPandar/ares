use std::num::{NonZeroU32, NonZeroUsize};

use crate::{
    ProjectObject, ProjectVolumeType, project::effective_config::types::ResolvedProjectObject,
};

use super::{
    closing::{PostClosingLayer, PostClosingPrintObject},
    layers::PlannedPrintObject,
};

const Z_EPSILON: f32 = 1e-4;

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
pub(super) struct BoundingBox3f {
    min: [f32; 3],
    max: [f32; 3],
}

impl BoundingBox3f {
    pub(super) const fn min(self) -> [f32; 3] {
        self.min
    }

    pub(super) const fn max(self) -> [f32; 3] {
        self.max
    }

    pub(super) fn contains_z(self, z: f32) -> bool {
        self.min[2] <= z && z <= self.max[2]
    }

    pub(super) fn intersects_xy(self, other: Self) -> bool {
        !(self.max[0] < other.min[0]
            || other.max[0] < self.min[0]
            || self.max[1] < other.min[1]
            || other.max[1] < self.min[1])
    }

    pub(super) fn intersects(self, other: Self) -> bool {
        self.intersects_xy(other) && !(self.max[2] < other.min[2] || other.max[2] < self.min[2])
    }

    pub(super) fn extend(&mut self, other: Self) {
        for axis in 0..3 {
            self.min[axis] = self.min[axis].min(other.min[axis]);
            self.max[axis] = self.max[axis].max(other.max[axis]);
        }
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
    let (plan, post_i_volumes) = post_i.into_parts();
    let object_transform = resolved.print_objects[0].transform;
    let xy_inflate = (resolved.object.xy_contour_compensation.0 as f32).max(0.0);
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
        let mesh = source_volume.mesh();
        let first_vertex = mesh.vertices()[mesh.triangles()[0][0] as usize];
        let first = transform.transform_point_f32(first_vertex);
        let mut min = first;
        let mut max = first;
        for &vertex_index in mesh.triangles().iter().flatten() {
            let point = transform.transform_point_f32(mesh.vertices()[vertex_index as usize]);
            for axis in 0..3 {
                min[axis] = min[axis].min(point[axis]);
                max[axis] = max[axis].max(point[axis]);
            }
        }
        min[0] -= xy_inflate;
        min[1] -= xy_inflate;
        max[0] += xy_inflate;
        max[1] += xy_inflate;
        min[2] -= Z_EPSILON;
        max[2] += Z_EPSILON;
        bounds.push(VolumeBound {
            source_volume_index,
            occurrence_id,
            bbox: BoundingBox3f { min, max },
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
