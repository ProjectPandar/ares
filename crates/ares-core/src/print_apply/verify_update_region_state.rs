use super::model_volume_state::StagedModelVolumeType;
use super::print_region_state::{StagedPrintRegionRefCount, staged_print_region_ref_reset};
use super::volume_cache_state::{StagedExtentBox, StagedVolumeExtents, staged_find_volume_extents};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedVerifyModelVolume {
    id: u64,
}

impl StagedVerifyModelVolume {
    pub(super) fn new(id: u64) -> Self {
        Self { id }
    }

    pub(super) fn id(&self) -> u64 {
        self.id
    }
}

pub(super) fn staged_verify_update_print_object_regions_init(
    model_volumes: &mut [StagedVerifyModelVolume],
    all_regions: &mut [StagedPrintRegionRefCount],
) {
    model_volumes.sort_by_key(|volume| volume.id);
    for region in all_regions {
        staged_print_region_ref_reset(region);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedVerifyVolumeRegion {
    volume_id: u64,
    volume_type: StagedModelVolumeType,
    parent: isize,
}

impl StagedVerifyVolumeRegion {
    pub(super) fn new(volume_id: u64, volume_type: StagedModelVolumeType) -> Self {
        Self::with_parent(volume_id, volume_type, -1)
    }

    pub(super) fn with_parent(
        volume_id: u64,
        volume_type: StagedModelVolumeType,
        parent: isize,
    ) -> Self {
        Self {
            volume_id,
            volume_type,
            parent,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedVerifyRegionMatch {
    region_id: usize,
    volume_id: u64,
    first_modifier_visit: bool,
}

impl StagedVerifyRegionMatch {
    pub(super) fn new(region_id: usize, volume_id: u64, first_modifier_visit: bool) -> Self {
        Self {
            region_id,
            volume_id,
            first_modifier_visit,
        }
    }
}

pub(super) fn staged_verify_update_volume_region_matches(
    model_volumes: &[StagedVerifyModelVolume],
    volume_regions: &[StagedVerifyVolumeRegion],
) -> Vec<StagedVerifyRegionMatch> {
    let mut matches = Vec::new();
    let mut last_modifier_volume_id = None;
    for (region_id, region) in volume_regions.iter().enumerate() {
        let is_modifier = region.volume_type == StagedModelVolumeType::ParameterModifier;
        if region.volume_type != StagedModelVolumeType::ModelPart && !is_modifier {
            continue;
        }
        let index = model_volumes.partition_point(|volume| volume.id < region.volume_id);
        assert!(index < model_volumes.len() && model_volumes[index].id == region.volume_id);

        let first_modifier_visit = is_modifier && last_modifier_volume_id != Some(region.volume_id);
        if is_modifier {
            last_modifier_volume_id = Some(region.volume_id);
        }
        matches.push(StagedVerifyRegionMatch::new(
            region_id,
            region.volume_id,
            first_modifier_visit,
        ));
    }
    matches
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct StagedVerifyParentScan {
    final_next_region_id: usize,
    scanned_parent_ids: Vec<usize>,
    existing_override_parent_ids: Vec<usize>,
}

impl StagedVerifyParentScan {
    pub(super) fn new(
        final_next_region_id: usize,
        scanned_parent_ids: Vec<usize>,
        existing_override_parent_ids: Vec<usize>,
    ) -> Self {
        Self {
            final_next_region_id,
            scanned_parent_ids,
            existing_override_parent_ids,
        }
    }
}

pub(super) fn staged_verify_update_modifier_parent_scan(
    volume_regions: &[StagedVerifyVolumeRegion],
    current_region_id: usize,
) -> StagedVerifyParentScan {
    let current = volume_regions[current_region_id];
    let mut next_region_id = current_region_id;
    let mut scanned_parent_ids = Vec::new();
    let mut existing_override_parent_ids = Vec::new();

    for parent_region_id in (0..current_region_id).rev() {
        let parent_region = volume_regions[parent_region_id];
        assert_ne!(parent_region.volume_id, current.volume_id);
        if parent_region.volume_type != StagedModelVolumeType::ModelPart
            && parent_region.volume_type != StagedModelVolumeType::ParameterModifier
        {
            continue;
        }
        assert!(
            next_region_id == volume_regions.len()
                || volume_regions[next_region_id].volume_id != current.volume_id
                || volume_regions[next_region_id].parent <= parent_region_id as isize
        );
        scanned_parent_ids.push(parent_region_id);
        if next_region_id < volume_regions.len()
            && volume_regions[next_region_id].volume_id == current.volume_id
            && volume_regions[next_region_id].parent == parent_region_id as isize
        {
            existing_override_parent_ids.push(parent_region_id);
            next_region_id += 1;
        }
    }

    StagedVerifyParentScan::new(
        next_region_id,
        scanned_parent_ids,
        existing_override_parent_ids,
    )
}

pub(super) fn staged_verify_update_current_modifier_bbox(
    volume_extents: &[StagedVolumeExtents],
    current_volume_id: u64,
) -> StagedExtentBox {
    *staged_find_volume_extents(volume_extents, current_volume_id).unwrap()
}
