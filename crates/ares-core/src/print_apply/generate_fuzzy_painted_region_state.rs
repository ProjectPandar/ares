use super::generate_fuzzy_volume_region_state::{
    StagedGenerateFuzzyConfig, StagedGenerateFuzzyParentType, StagedGenerateFuzzyRegion,
    StagedGenerateFuzzySkinType,
};
use super::generate_regions_state::{StagedGeneratePrintObjectRegions, StagedGenerateRegionSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedGenerateFuzzyPaintedParentRegion {
    parent_config_marker: u64,
    fuzzy_skin: StagedGenerateFuzzySkinType,
}

impl StagedGenerateFuzzyPaintedParentRegion {
    pub(super) fn new(parent_config_marker: u64, fuzzy_skin: StagedGenerateFuzzySkinType) -> Self {
        Self {
            parent_config_marker,
            fuzzy_skin,
        }
    }
}

pub(super) fn staged_generate_fuzzy_painted_regions(
    shell: &mut StagedGeneratePrintObjectRegions,
    has_painted_fuzzy_skin: bool,
    parent_regions: &[StagedGenerateFuzzyPaintedParentRegion],
    region_set: &mut StagedGenerateRegionSet,
) -> Vec<StagedGenerateFuzzyRegion> {
    if !has_painted_fuzzy_skin {
        return Vec::new();
    }

    let mut fuzzy_regions = Vec::new();
    for (parent_id, parent_region) in parent_regions.iter().enumerate() {
        let derived_config = StagedGenerateFuzzyConfig::from_parent(
            parent_region.parent_config_marker,
            parent_region.fuzzy_skin,
        );
        let region_id = region_set.get_create_region(shell, derived_config.region_key());
        fuzzy_regions.push(StagedGenerateFuzzyRegion::new(
            StagedGenerateFuzzyParentType::PaintedRegion,
            parent_id,
            region_id,
            derived_config,
        ));
    }

    fuzzy_regions
}
