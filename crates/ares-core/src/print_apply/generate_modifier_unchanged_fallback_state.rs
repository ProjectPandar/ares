use super::generate_regions_state::StagedGenerateRegionConfigKey;
use super::model_volume_state::StagedModelVolumeType;
use super::volume_cache_state::StagedExtentBox;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct StagedGenerateModifierFallbackCurrent {
    id: u64,
    volume_type: StagedModelVolumeType,
    bbox: StagedExtentBox,
}

impl StagedGenerateModifierFallbackCurrent {
    pub(super) fn new(id: u64, volume_type: StagedModelVolumeType, bbox: StagedExtentBox) -> Self {
        Self {
            id,
            volume_type,
            bbox,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedGenerateModifierFallbackCandidate {
    parent_region_id: usize,
    parent_volume_type: StagedModelVolumeType,
    region_id: u64,
    parent_config: StagedGenerateRegionConfigKey,
    derived_config: StagedGenerateRegionConfigKey,
}

impl StagedGenerateModifierFallbackCandidate {
    pub(super) fn new(
        parent_region_id: usize,
        parent_volume_type: StagedModelVolumeType,
        region_id: u64,
        parent_config: StagedGenerateRegionConfigKey,
        derived_config: StagedGenerateRegionConfigKey,
    ) -> Self {
        Self {
            parent_region_id,
            parent_volume_type,
            region_id,
            parent_config,
            derived_config,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct StagedGenerateModifierFallbackRegion {
    model_volume_id: u64,
    parent: usize,
    region_id: u64,
    bbox: StagedExtentBox,
}

impl StagedGenerateModifierFallbackRegion {
    pub(super) fn model_volume_id(&self) -> u64 {
        self.model_volume_id
    }

    pub(super) fn parent(&self) -> usize {
        self.parent
    }

    pub(super) fn region_id(&self) -> u64 {
        self.region_id
    }

    pub(super) fn bbox(&self) -> StagedExtentBox {
        self.bbox
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct StagedGenerateModifierFallbackResult {
    parent_model_part_id: Option<usize>,
    volume_region: Option<StagedGenerateModifierFallbackRegion>,
}

impl StagedGenerateModifierFallbackResult {
    fn new(
        parent_model_part_id: Option<usize>,
        volume_region: Option<StagedGenerateModifierFallbackRegion>,
    ) -> Self {
        Self {
            parent_model_part_id,
            volume_region,
        }
    }

    pub(super) fn parent_model_part_id(&self) -> Option<usize> {
        self.parent_model_part_id
    }

    pub(super) fn volume_region(&self) -> Option<StagedGenerateModifierFallbackRegion> {
        self.volume_region
    }
}

pub(super) fn staged_generate_modifier_unchanged_fallback(
    current: StagedGenerateModifierFallbackCurrent,
    candidates: &[StagedGenerateModifierFallbackCandidate],
    changed_added: bool,
) -> StagedGenerateModifierFallbackResult {
    if current.volume_type != StagedModelVolumeType::ParameterModifier {
        return StagedGenerateModifierFallbackResult::new(None, None);
    }

    let selected = candidates.iter().find(|candidate| {
        candidate.parent_volume_type == StagedModelVolumeType::ModelPart
            && candidate.derived_config == candidate.parent_config
    });
    let parent_model_part_id = selected.map(|candidate| candidate.parent_region_id);
    let volume_region = if changed_added {
        None
    } else {
        selected.map(|candidate| StagedGenerateModifierFallbackRegion {
            model_volume_id: current.id,
            parent: candidate.parent_region_id,
            region_id: candidate.region_id,
            bbox: current.bbox,
        })
    };

    StagedGenerateModifierFallbackResult::new(parent_model_part_id, volume_region)
}
