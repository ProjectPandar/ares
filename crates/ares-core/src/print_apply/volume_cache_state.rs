use super::mesh_state::{StagedLayerHeightRange, StagedRangeBoundingBox3f};
use super::model_volume_state::{StagedModelVolumeType, staged_model_volume_solid_or_modifier};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct StagedVolumeCacheRegions {
    all_regions: Vec<u64>,
    cached_volume_ids: Vec<u64>,
}

impl StagedVolumeCacheRegions {
    pub(super) fn new(all_regions: Vec<u64>, cached_volume_ids: Vec<u64>) -> Self {
        Self {
            all_regions,
            cached_volume_ids,
        }
    }

    pub(super) fn all_regions(&self) -> &[u64] {
        &self.all_regions
    }

    pub(super) fn cached_volume_ids(&self) -> &[u64] {
        &self.cached_volume_ids
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedCachedModelVolume {
    id: u64,
    volume_type: StagedModelVolumeType,
    transform_key: u64,
}

impl StagedCachedModelVolume {
    pub(super) fn new(id: u64, volume_type: StagedModelVolumeType, transform_key: u64) -> Self {
        Self {
            id,
            volume_type,
            transform_key,
        }
    }
}

pub(super) fn staged_print_objects_regions_invalidate_keep_some_volumes(
    regions: &mut StagedVolumeCacheRegions,
    old_volumes: &[StagedCachedModelVolume],
    new_volumes: &[StagedCachedModelVolume],
) {
    regions.all_regions.clear();

    let mut old_volumes = old_volumes.to_vec();
    let mut new_volumes = new_volumes.to_vec();
    old_volumes.sort_by_key(|volume| volume.id);
    new_volumes.sort_by_key(|volume| volume.id);

    let mut cached_index = 0;
    let mut kept_count = 0;
    let mut old_index = 0;
    for new_volume in &new_volumes {
        if !staged_model_volume_solid_or_modifier(new_volume.volume_type) {
            continue;
        }
        while old_index < old_volumes.len() && old_volumes[old_index].id < new_volume.id {
            old_index += 1;
        }
        if old_index == old_volumes.len() || old_volumes[old_index].id != new_volume.id {
            continue;
        }
        if old_volumes[old_index].transform_key == new_volume.transform_key {
            while regions.cached_volume_ids[cached_index] < old_volumes[old_index].id {
                cached_index += 1;
                assert!(cached_index < regions.cached_volume_ids.len());
            }
            assert_eq!(
                regions.cached_volume_ids[cached_index],
                old_volumes[old_index].id
            );
            regions.cached_volume_ids[kept_count] = regions.cached_volume_ids[cached_index];
            kept_count += 1;
            cached_index += 1;
        }
    }
    regions.cached_volume_ids.truncate(kept_count);
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct StagedExtentBox {
    min: [f32; 3],
    max: [f32; 3],
}

impl StagedExtentBox {
    pub(super) fn new(min: [f32; 3], max: [f32; 3]) -> Self {
        Self { min, max }
    }

    pub(super) fn min(&self) -> [f32; 3] {
        self.min
    }

    pub(super) fn max(&self) -> [f32; 3] {
        self.max
    }

    pub(super) fn extend_box(&mut self, other: &Self) {
        for axis in 0..3 {
            self.min[axis] = self.min[axis].min(other.min[axis]);
            self.max[axis] = self.max[axis].max(other.max[axis]);
        }
    }

    pub(super) fn intersects(&self, other: &Self) -> bool {
        (0..3).all(|axis| self.min[axis] <= other.max[axis] && other.min[axis] <= self.max[axis])
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct StagedVolumeExtents {
    volume_id: u64,
    bbox: StagedExtentBox,
}

impl StagedVolumeExtents {
    pub(super) fn new(volume_id: u64, bbox: StagedExtentBox) -> Self {
        Self { volume_id, bbox }
    }
}

pub(super) fn staged_find_volume_extents(
    extents: &[StagedVolumeExtents],
    volume_id: u64,
) -> Option<&StagedExtentBox> {
    let index = extents.partition_point(|extent| extent.volume_id < volume_id);
    if index < extents.len() && extents[index].volume_id == volume_id {
        Some(&extents[index].bbox)
    } else {
        None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedVolumeRegion {
    volume_id: u64,
    is_model_part: bool,
    parent: isize,
}

impl StagedVolumeRegion {
    pub(super) fn new(volume_id: u64, is_model_part: bool, parent: isize) -> Self {
        Self {
            volume_id,
            is_model_part,
            parent,
        }
    }
}

pub(super) fn staged_find_modifier_volume_extents(
    volume_regions: &[StagedVolumeRegion],
    volume_extents: &[StagedVolumeExtents],
    this_region_id: usize,
) -> StagedExtentBox {
    let this_region = volume_regions[this_region_id];
    let mut out = *staged_find_volume_extents(volume_extents, this_region.volume_id).unwrap();
    if !this_region.is_model_part {
        let mut parent_region_id = this_region.parent;
        loop {
            assert!(parent_region_id >= 0);
            let parent_region = volume_regions[parent_region_id as usize];
            let parent_extents =
                staged_find_volume_extents(volume_extents, parent_region.volume_id).unwrap();
            out.extend_box(parent_extents);
            if parent_region.is_model_part {
                break;
            }
            parent_region_id = parent_region.parent;
        }
    }
    out
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct StagedParentBboxIntersectionGate {
    parent_region_id: usize,
    parent_bbox: StagedExtentBox,
    current_modifier_bbox: StagedExtentBox,
    intersects: bool,
}

impl StagedParentBboxIntersectionGate {
    pub(super) fn new(
        parent_region_id: usize,
        parent_bbox: StagedExtentBox,
        current_modifier_bbox: StagedExtentBox,
        intersects: bool,
    ) -> Self {
        Self {
            parent_region_id,
            parent_bbox,
            current_modifier_bbox,
            intersects,
        }
    }
}

pub(super) fn staged_verify_update_parent_bbox_intersection_gate(
    volume_regions: &[StagedVolumeRegion],
    volume_extents: &[StagedVolumeExtents],
    current_modifier_bbox: StagedExtentBox,
    parent_region_id: usize,
) -> StagedParentBboxIntersectionGate {
    let parent_bbox =
        staged_find_modifier_volume_extents(volume_regions, volume_extents, parent_region_id);
    let intersects = parent_bbox.intersects(&current_modifier_bbox);
    StagedParentBboxIntersectionGate::new(
        parent_region_id,
        parent_bbox,
        current_modifier_bbox,
        intersects,
    )
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct StagedMultiLayerVolumeCacheLayer {
    volumes: Vec<StagedVolumeExtents>,
}

impl StagedMultiLayerVolumeCacheLayer {
    pub(super) fn new(volumes: Vec<StagedVolumeExtents>) -> Self {
        Self { volumes }
    }

    pub(super) fn volumes(&self) -> &[StagedVolumeExtents] {
        &self.volumes
    }
}

pub(super) fn staged_update_volume_bboxes_multi_layer_old_extents(
    cached_volume_ids: &[u64],
    layer_ranges: &mut [StagedMultiLayerVolumeCacheLayer],
) -> Vec<Vec<StagedVolumeExtents>> {
    if cached_volume_ids.is_empty() {
        for layer in layer_ranges {
            layer.volumes.clear();
        }
        Vec::new()
    } else {
        layer_ranges
            .iter_mut()
            .map(|layer| std::mem::take(&mut layer.volumes))
            .collect()
    }
}

pub(super) fn staged_update_volume_bboxes_multi_layer_expanded_ranges(
    layer_ranges: &[StagedLayerHeightRange],
    epsilon: f64,
) -> Vec<StagedLayerHeightRange> {
    layer_ranges
        .iter()
        .map(|range| StagedLayerHeightRange::new(range.first() - epsilon, range.second() + epsilon))
        .collect()
}

pub(super) fn staged_update_volume_bboxes_multi_layer_cached_reuse(
    cached_volume_ids: &[u64],
    old_extents_by_layer: &[Vec<StagedVolumeExtents>],
    layer_ranges: &mut [StagedMultiLayerVolumeCacheLayer],
    model_volumes: &[StagedUpdateVolumeBboxesVolume],
) {
    for volume in model_volumes {
        if !staged_model_volume_solid_or_modifier(volume.volume_type) {
            continue;
        }
        if cached_volume_ids.binary_search(&volume.id).is_err() {
            continue;
        }
        for (layer, old_extents) in layer_ranges.iter_mut().zip(old_extents_by_layer) {
            if let Some(bbox) = staged_find_volume_extents(old_extents, volume.id) {
                layer
                    .volumes
                    .push(StagedVolumeExtents::new(volume.id, *bbox));
            }
        }
    }
}

pub(super) fn staged_update_volume_bboxes_multi_layer_uncached_insertion(
    cached_volume_ids: &[u64],
    bboxes_by_volume: &[Vec<StagedRangeBoundingBox3f>],
    layer_ranges: &mut [StagedMultiLayerVolumeCacheLayer],
    model_volumes: &[StagedUpdateVolumeBboxesVolume],
) {
    for (volume_index, volume) in model_volumes.iter().enumerate() {
        if !staged_model_volume_solid_or_modifier(volume.volume_type)
            || cached_volume_ids.binary_search(&volume.id).is_ok()
        {
            continue;
        }

        for (layer, bbox) in layer_ranges.iter_mut().zip(&bboxes_by_volume[volume_index]) {
            if bbox.is_populated() {
                layer.volumes.push(StagedVolumeExtents::new(
                    volume.id,
                    StagedExtentBox::new(bbox.min(), bbox.max()),
                ));
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedUpdateVolumeBboxesVolume {
    id: u64,
    volume_type: StagedModelVolumeType,
}

impl StagedUpdateVolumeBboxesVolume {
    pub(super) fn new(id: u64, volume_type: StagedModelVolumeType) -> Self {
        Self { id, volume_type }
    }
}

pub(super) fn staged_update_volume_bboxes_volume_order_cache_ids(
    cached_volume_ids: &mut Vec<u64>,
    model_volumes: &[StagedUpdateVolumeBboxesVolume],
) -> Vec<u64> {
    let mut model_volumes = model_volumes.to_vec();
    model_volumes.sort_by_key(|volume| volume.id);

    let sorted_ids = model_volumes
        .iter()
        .filter(|volume| staged_model_volume_solid_or_modifier(volume.volume_type))
        .map(|volume| volume.id)
        .collect::<Vec<_>>();

    cached_volume_ids.clear();
    cached_volume_ids.reserve(model_volumes.len());
    cached_volume_ids.extend(sorted_ids.iter().copied());

    sorted_ids
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct StagedSingleLayerVolumeBboxInput {
    id: u64,
    volume_type: StagedModelVolumeType,
    bbox: StagedExtentBox,
}

impl StagedSingleLayerVolumeBboxInput {
    pub(super) fn new(id: u64, volume_type: StagedModelVolumeType, bbox: StagedExtentBox) -> Self {
        Self {
            id,
            volume_type,
            bbox,
        }
    }
}

pub(super) fn staged_update_volume_bboxes_single_layer(
    cached_volume_ids: &[u64],
    old_extents: &[StagedVolumeExtents],
    model_volumes: &[StagedSingleLayerVolumeBboxInput],
) -> Vec<StagedVolumeExtents> {
    let mut extents = Vec::with_capacity(model_volumes.len());
    for volume in model_volumes {
        if !staged_model_volume_solid_or_modifier(volume.volume_type) {
            continue;
        }
        if cached_volume_ids.binary_search(&volume.id).is_ok() {
            if let Some(bbox) = staged_find_volume_extents(old_extents, volume.id) {
                extents.push(StagedVolumeExtents::new(volume.id, *bbox));
            }
        } else {
            extents.push(StagedVolumeExtents::new(volume.id, volume.bbox));
        }
    }
    extents
}
