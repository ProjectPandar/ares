use super::generate_fuzzy_volume_region_state::{
    StagedGenerateFuzzyParentType, StagedGenerateFuzzyRegion,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedGenerateFuzzySortParent {
    print_object_region_id: u64,
}

impl StagedGenerateFuzzySortParent {
    pub(super) fn new(print_object_region_id: u64) -> Self {
        Self {
            print_object_region_id,
        }
    }
}

pub(super) fn staged_sort_generate_fuzzy_painted_regions(
    volume_parents: &[StagedGenerateFuzzySortParent],
    painted_parents: &[StagedGenerateFuzzySortParent],
    fuzzy_regions: &[StagedGenerateFuzzyRegion],
) -> Vec<StagedGenerateFuzzyRegion> {
    let mut sorted = fuzzy_regions.to_vec();
    sorted.sort_by_key(|region| match region.parent_type() {
        StagedGenerateFuzzyParentType::VolumeRegion => {
            volume_parents[region.parent()].print_object_region_id
        }
        StagedGenerateFuzzyParentType::PaintedRegion => {
            painted_parents[region.parent()].print_object_region_id
        }
    });
    sorted
}
