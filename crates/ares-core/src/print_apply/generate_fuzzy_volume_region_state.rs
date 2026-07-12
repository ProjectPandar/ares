use super::generate_regions_state::{
    StagedGeneratePrintObjectRegions, StagedGenerateRegionConfigKey, StagedGenerateRegionSet,
};
use super::model_volume_state::StagedModelVolumeType;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum StagedGenerateFuzzySkinType {
    DisabledFuzzy,
    Contour,
    External,
    Hole,
    AllWalls,
    All,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedGenerateFuzzyParentVolumeRegion {
    volume_type: StagedModelVolumeType,
    parent_config_marker: u64,
    fuzzy_skin: StagedGenerateFuzzySkinType,
}

impl StagedGenerateFuzzyParentVolumeRegion {
    pub(super) fn new(
        volume_type: StagedModelVolumeType,
        parent_config_marker: u64,
        fuzzy_skin: StagedGenerateFuzzySkinType,
    ) -> Self {
        Self {
            volume_type,
            parent_config_marker,
            fuzzy_skin,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedGenerateFuzzyConfig {
    marker: u64,
    fuzzy_skin: StagedGenerateFuzzySkinType,
}

impl StagedGenerateFuzzyConfig {
    pub(super) fn from_parent(parent_marker: u64, fuzzy_skin: StagedGenerateFuzzySkinType) -> Self {
        let fuzzy_skin = match fuzzy_skin {
            StagedGenerateFuzzySkinType::DisabledFuzzy => {
                StagedGenerateFuzzySkinType::DisabledFuzzy
            }
            StagedGenerateFuzzySkinType::Contour
            | StagedGenerateFuzzySkinType::External
            | StagedGenerateFuzzySkinType::Hole
            | StagedGenerateFuzzySkinType::AllWalls
            | StagedGenerateFuzzySkinType::All => StagedGenerateFuzzySkinType::All,
        };
        Self {
            marker: parent_marker,
            fuzzy_skin,
        }
    }

    pub(super) fn marker(&self) -> u64 {
        self.marker
    }

    pub(super) fn fuzzy_skin(&self) -> StagedGenerateFuzzySkinType {
        self.fuzzy_skin
    }

    pub(super) fn region_key(&self) -> StagedGenerateRegionConfigKey {
        let fuzzy_ordinal = match self.fuzzy_skin {
            StagedGenerateFuzzySkinType::DisabledFuzzy => 0,
            StagedGenerateFuzzySkinType::Contour => 1,
            StagedGenerateFuzzySkinType::External => 2,
            StagedGenerateFuzzySkinType::Hole => 3,
            StagedGenerateFuzzySkinType::AllWalls => 4,
            StagedGenerateFuzzySkinType::All => 5,
        };
        StagedGenerateRegionConfigKey::new(self.marker, fuzzy_ordinal)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum StagedGenerateFuzzyParentType {
    VolumeRegion,
    PaintedRegion,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedGenerateFuzzyRegion {
    parent_type: StagedGenerateFuzzyParentType,
    parent: usize,
    region_id: u64,
    derived_config: StagedGenerateFuzzyConfig,
}

impl StagedGenerateFuzzyRegion {
    pub(super) fn new(
        parent_type: StagedGenerateFuzzyParentType,
        parent: usize,
        region_id: u64,
        derived_config: StagedGenerateFuzzyConfig,
    ) -> Self {
        Self {
            parent_type,
            parent,
            region_id,
            derived_config,
        }
    }

    pub(super) fn parent_type(&self) -> StagedGenerateFuzzyParentType {
        self.parent_type
    }

    pub(super) fn parent(&self) -> usize {
        self.parent
    }

    pub(super) fn region_id(&self) -> u64 {
        self.region_id
    }

    pub(super) fn derived_config(&self) -> StagedGenerateFuzzyConfig {
        self.derived_config
    }
}

pub(super) fn staged_generate_fuzzy_volume_regions(
    shell: &mut StagedGeneratePrintObjectRegions,
    has_painted_fuzzy_skin: bool,
    parent_regions: &[StagedGenerateFuzzyParentVolumeRegion],
    region_set: &mut StagedGenerateRegionSet,
) -> Vec<StagedGenerateFuzzyRegion> {
    if !has_painted_fuzzy_skin {
        return Vec::new();
    }

    let mut fuzzy_regions = Vec::new();
    for (parent_id, parent_region) in parent_regions.iter().enumerate() {
        if parent_region.volume_type != StagedModelVolumeType::ModelPart
            && parent_region.volume_type != StagedModelVolumeType::ParameterModifier
        {
            continue;
        }

        let derived_config = StagedGenerateFuzzyConfig::from_parent(
            parent_region.parent_config_marker,
            parent_region.fuzzy_skin,
        );
        let region_id = region_set.get_create_region(shell, derived_config.region_key());
        fuzzy_regions.push(StagedGenerateFuzzyRegion::new(
            StagedGenerateFuzzyParentType::VolumeRegion,
            parent_id,
            region_id,
            derived_config,
        ));
    }

    fuzzy_regions
}
