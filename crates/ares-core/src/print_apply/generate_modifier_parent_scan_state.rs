use super::model_volume_state::StagedModelVolumeType;
use super::volume_cache_state::{
    StagedExtentBox, StagedVolumeExtents, StagedVolumeRegion, staged_find_modifier_volume_extents,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedGenerateModifierParentRegion {
    volume_id: u64,
    volume_type: StagedModelVolumeType,
    parent: isize,
}

impl StagedGenerateModifierParentRegion {
    pub(super) fn new(volume_id: u64, volume_type: StagedModelVolumeType, parent: isize) -> Self {
        Self {
            volume_id,
            volume_type,
            parent,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct StagedGenerateModifierParentScanInput {
    current_volume_type: StagedModelVolumeType,
    current_bbox: StagedExtentBox,
}

impl StagedGenerateModifierParentScanInput {
    pub(super) fn new(
        _current_volume_id: u64,
        current_volume_type: StagedModelVolumeType,
        current_bbox: StagedExtentBox,
    ) -> Self {
        Self {
            current_volume_type,
            current_bbox,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct StagedGenerateModifierParentCandidate {
    parent_region_id: usize,
    parent_bbox: StagedExtentBox,
}

impl StagedGenerateModifierParentCandidate {
    pub(super) fn parent_region_id(&self) -> usize {
        self.parent_region_id
    }
    pub(super) fn parent_bbox(&self) -> StagedExtentBox {
        self.parent_bbox
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct StagedGenerateModifierParentScan {
    added: bool,
    parent_model_part_id: isize,
    scanned_parent_ids: Vec<usize>,
    intersecting_parents: Vec<StagedGenerateModifierParentCandidate>,
}

impl StagedGenerateModifierParentScan {
    fn initial() -> Self {
        Self {
            added: false,
            parent_model_part_id: -1,
            scanned_parent_ids: Vec::new(),
            intersecting_parents: Vec::new(),
        }
    }
    pub(super) fn added(&self) -> bool {
        self.added
    }
    pub(super) fn parent_model_part_id(&self) -> isize {
        self.parent_model_part_id
    }
    pub(super) fn scanned_parent_ids(&self) -> &[usize] {
        &self.scanned_parent_ids
    }
    pub(super) fn intersecting_parents(&self) -> &[StagedGenerateModifierParentCandidate] {
        &self.intersecting_parents
    }
}

pub(super) fn staged_generate_modifier_parent_scan(
    input: StagedGenerateModifierParentScanInput,
    parent_regions: &[StagedGenerateModifierParentRegion],
    volume_extents: &[StagedVolumeExtents],
) -> StagedGenerateModifierParentScan {
    let mut scan = StagedGenerateModifierParentScan::initial();
    if input.current_volume_type != StagedModelVolumeType::ParameterModifier {
        return scan;
    }
    let adapted: Vec<StagedVolumeRegion> = parent_regions
        .iter()
        .map(|r| {
            StagedVolumeRegion::new(
                r.volume_id,
                r.volume_type == StagedModelVolumeType::ModelPart,
                r.parent,
            )
        })
        .collect();
    for parent_region_id in (0..parent_regions.len()).rev() {
        let r = parent_regions[parent_region_id];
        if r.volume_type != StagedModelVolumeType::ModelPart
            && r.volume_type != StagedModelVolumeType::ParameterModifier
        {
            continue;
        }
        scan.scanned_parent_ids.push(parent_region_id);
        let parent_bbox =
            staged_find_modifier_volume_extents(&adapted, volume_extents, parent_region_id);
        if parent_bbox.intersects(&input.current_bbox) {
            scan.intersecting_parents
                .push(StagedGenerateModifierParentCandidate {
                    parent_region_id,
                    parent_bbox,
                });
        }
    }
    scan
}
