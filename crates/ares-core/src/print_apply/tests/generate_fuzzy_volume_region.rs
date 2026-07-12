use super::super::generate_fuzzy_volume_region_state::{
    StagedGenerateFuzzyParentType, StagedGenerateFuzzyParentVolumeRegion,
    StagedGenerateFuzzySkinType, staged_generate_fuzzy_volume_regions,
};
use super::super::generate_regions_state::{
    StagedGeneratePrintObjectRegions, StagedGenerateRegionSet,
};
use super::super::model_volume_state::StagedModelVolumeType;

fn parent(
    volume_type: StagedModelVolumeType,
    marker: u64,
    fuzzy_skin: StagedGenerateFuzzySkinType,
) -> StagedGenerateFuzzyParentVolumeRegion {
    StagedGenerateFuzzyParentVolumeRegion::new(volume_type, marker, fuzzy_skin)
}

#[test]
fn generate_fuzzy_volume_region_disabled_gate_is_noop() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut region_set = StagedGenerateRegionSet::new();

    let regions = staged_generate_fuzzy_volume_regions(
        &mut shell,
        false,
        &[parent(
            StagedModelVolumeType::ModelPart,
            10,
            StagedGenerateFuzzySkinType::Contour,
        )],
        &mut region_set,
    );

    assert!(regions.is_empty());
    assert!(shell.all_regions().is_empty());
}

#[test]
fn generate_fuzzy_volume_region_appends_for_model_part_and_modifier_parents() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut region_set = StagedGenerateRegionSet::new();

    let regions = staged_generate_fuzzy_volume_regions(
        &mut shell,
        true,
        &[
            parent(
                StagedModelVolumeType::ModelPart,
                10,
                StagedGenerateFuzzySkinType::Contour,
            ),
            parent(
                StagedModelVolumeType::ParameterModifier,
                20,
                StagedGenerateFuzzySkinType::DisabledFuzzy,
            ),
        ],
        &mut region_set,
    );

    assert_eq!(regions.len(), 2);
    assert_eq!(
        regions[0].parent_type(),
        StagedGenerateFuzzyParentType::VolumeRegion
    );
    assert_eq!(regions[0].parent(), 0);
    assert_eq!(regions[0].derived_config().marker(), 10);
    assert_eq!(
        regions[1].parent_type(),
        StagedGenerateFuzzyParentType::VolumeRegion
    );
    assert_eq!(regions[1].parent(), 1);
    assert_eq!(regions[1].derived_config().marker(), 20);
    assert_eq!(shell.all_regions().len(), 2);
}

#[test]
fn generate_fuzzy_volume_region_skips_ineligible_parent_types() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut region_set = StagedGenerateRegionSet::new();

    let regions = staged_generate_fuzzy_volume_regions(
        &mut shell,
        true,
        &[
            parent(
                StagedModelVolumeType::NegativeVolume,
                10,
                StagedGenerateFuzzySkinType::Contour,
            ),
            parent(
                StagedModelVolumeType::SupportBlocker,
                20,
                StagedGenerateFuzzySkinType::All,
            ),
            parent(
                StagedModelVolumeType::ModelPart,
                30,
                StagedGenerateFuzzySkinType::All,
            ),
        ],
        &mut region_set,
    );

    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].parent(), 2);
    assert_eq!(regions[0].derived_config().marker(), 30);
    assert_eq!(shell.all_regions().len(), 1);
}

#[test]
fn generate_fuzzy_volume_region_converts_non_disabled_to_all() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut region_set = StagedGenerateRegionSet::new();

    let regions = staged_generate_fuzzy_volume_regions(
        &mut shell,
        true,
        &[parent(
            StagedModelVolumeType::ModelPart,
            10,
            StagedGenerateFuzzySkinType::Contour,
        )],
        &mut region_set,
    );

    assert_eq!(
        regions[0].derived_config().fuzzy_skin(),
        StagedGenerateFuzzySkinType::All
    );
}

#[test]
fn generate_fuzzy_volume_region_preserves_disabled_fuzzy() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut region_set = StagedGenerateRegionSet::new();

    let regions = staged_generate_fuzzy_volume_regions(
        &mut shell,
        true,
        &[parent(
            StagedModelVolumeType::ModelPart,
            10,
            StagedGenerateFuzzySkinType::DisabledFuzzy,
        )],
        &mut region_set,
    );

    assert_eq!(
        regions[0].derived_config().fuzzy_skin(),
        StagedGenerateFuzzySkinType::DisabledFuzzy
    );
}

#[test]
fn generate_fuzzy_volume_region_reuses_equal_derived_configs() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut region_set = StagedGenerateRegionSet::new();

    let regions = staged_generate_fuzzy_volume_regions(
        &mut shell,
        true,
        &[
            parent(
                StagedModelVolumeType::ModelPart,
                10,
                StagedGenerateFuzzySkinType::Contour,
            ),
            parent(
                StagedModelVolumeType::ParameterModifier,
                10,
                StagedGenerateFuzzySkinType::All,
            ),
        ],
        &mut region_set,
    );

    assert_eq!(regions[0].region_id(), 0);
    assert_eq!(regions[1].region_id(), 0);
    assert_eq!(shell.all_regions().len(), 1);
}

#[test]
fn generate_fuzzy_volume_region_empty_parents_is_noop() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut region_set = StagedGenerateRegionSet::new();

    let regions = staged_generate_fuzzy_volume_regions(&mut shell, true, &[], &mut region_set);

    assert!(regions.is_empty());
    assert!(shell.all_regions().is_empty());
}
