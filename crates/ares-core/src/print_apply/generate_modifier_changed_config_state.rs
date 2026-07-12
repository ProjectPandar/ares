use super::generate_regions_state::{
    StagedGeneratePrintObjectRegions, StagedGenerateRegionConfigKey, StagedGenerateRegionSet,
};
use super::model_volume_state::StagedModelVolumeType;
use super::volume_cache_state::StagedExtentBox;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct StagedGenerateModifierCurrent {
    id: u64,
    volume_type: StagedModelVolumeType,
    bbox: StagedExtentBox,
}

impl StagedGenerateModifierCurrent {
    pub(super) fn new(id: u64, volume_type: StagedModelVolumeType, bbox: StagedExtentBox) -> Self {
        Self {
            id,
            volume_type,
            bbox,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedGenerateModifierChangedCandidate {
    parent_region_id: usize,
    parent_config: StagedGenerateRegionConfigKey,
    derived_config: StagedGenerateRegionConfigKey,
}

impl StagedGenerateModifierChangedCandidate {
    pub(super) fn new(
        parent_region_id: usize,
        parent_config: StagedGenerateRegionConfigKey,
        derived_config: StagedGenerateRegionConfigKey,
    ) -> Self {
        Self {
            parent_region_id,
            parent_config,
            derived_config,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct StagedGenerateModifierChangedRegion {
    model_volume_id: u64,
    parent: usize,
    region_id: u64,
    bbox: StagedExtentBox,
}

impl StagedGenerateModifierChangedRegion {
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

#[derive(Clone, Debug, PartialEq)]
pub(super) struct StagedGenerateModifierChangedResult {
    added: bool,
    volume_regions: Vec<StagedGenerateModifierChangedRegion>,
}

impl StagedGenerateModifierChangedResult {
    fn new(volume_regions: Vec<StagedGenerateModifierChangedRegion>) -> Self {
        Self {
            added: !volume_regions.is_empty(),
            volume_regions,
        }
    }

    pub(super) fn added(&self) -> bool {
        self.added
    }

    pub(super) fn volume_regions(&self) -> &[StagedGenerateModifierChangedRegion] {
        &self.volume_regions
    }
}

pub(super) fn staged_generate_modifier_changed_config_regions(
    shell: &mut StagedGeneratePrintObjectRegions,
    current: StagedGenerateModifierCurrent,
    candidates: &[StagedGenerateModifierChangedCandidate],
    region_set: &mut StagedGenerateRegionSet,
) -> StagedGenerateModifierChangedResult {
    if current.volume_type != StagedModelVolumeType::ParameterModifier {
        return StagedGenerateModifierChangedResult::new(Vec::new());
    }

    let volume_regions = candidates
        .iter()
        .filter(|candidate| candidate.derived_config != candidate.parent_config)
        .map(|candidate| {
            let region_id = region_set.get_create_region(shell, candidate.derived_config);
            StagedGenerateModifierChangedRegion {
                model_volume_id: current.id,
                parent: candidate.parent_region_id,
                region_id,
                bbox: current.bbox,
            }
        })
        .collect();

    StagedGenerateModifierChangedResult::new(volume_regions)
}
