use super::super::generate_fuzzy_painted_region_state::{
    StagedGenerateFuzzyPaintedParentRegion, staged_generate_fuzzy_painted_regions,
};
use super::super::generate_fuzzy_volume_region_state::{
    StagedGenerateFuzzyParentType, StagedGenerateFuzzyParentVolumeRegion,
    StagedGenerateFuzzySkinType, staged_generate_fuzzy_volume_regions,
};
use super::super::generate_regions_state::{
    StagedGeneratePrintObjectRegions, StagedGenerateRegionSet,
};
use super::super::model_volume_state::StagedModelVolumeType;

fn parent(
    marker: u64,
    fuzzy_skin: StagedGenerateFuzzySkinType,
) -> StagedGenerateFuzzyPaintedParentRegion {
    StagedGenerateFuzzyPaintedParentRegion::new(marker, fuzzy_skin)
}

#[test]
fn generate_fuzzy_painted_region_disabled_gate_is_noop() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut region_set = StagedGenerateRegionSet::new();

    let regions = staged_generate_fuzzy_painted_regions(
        &mut shell,
        false,
        &[parent(10, StagedGenerateFuzzySkinType::External)],
        &mut region_set,
    );

    assert!(regions.is_empty());
    assert!(shell.all_regions().is_empty());
}

#[test]
fn generate_fuzzy_painted_region_appends_painted_parents_in_order() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut region_set = StagedGenerateRegionSet::new();

    let regions = staged_generate_fuzzy_painted_regions(
        &mut shell,
        true,
        &[
            parent(10, StagedGenerateFuzzySkinType::External),
            parent(20, StagedGenerateFuzzySkinType::AllWalls),
        ],
        &mut region_set,
    );

    assert_eq!(regions.len(), 2);
    assert_eq!(
        regions[0].parent_type(),
        StagedGenerateFuzzyParentType::PaintedRegion
    );
    assert_eq!(regions[0].parent(), 0);
    assert_eq!(regions[0].derived_config().marker(), 10);
    assert_eq!(
        regions[1].parent_type(),
        StagedGenerateFuzzyParentType::PaintedRegion
    );
    assert_eq!(regions[1].parent(), 1);
    assert_eq!(regions[1].derived_config().marker(), 20);
    assert_eq!(shell.all_regions().len(), 2);
}

#[test]
fn generate_fuzzy_painted_region_converts_non_disabled_to_all() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut region_set = StagedGenerateRegionSet::new();

    let regions = staged_generate_fuzzy_painted_regions(
        &mut shell,
        true,
        &[
            parent(10, StagedGenerateFuzzySkinType::Contour),
            parent(20, StagedGenerateFuzzySkinType::Hole),
        ],
        &mut region_set,
    );

    assert_eq!(
        regions[0].derived_config().fuzzy_skin(),
        StagedGenerateFuzzySkinType::All
    );
    assert_eq!(
        regions[1].derived_config().fuzzy_skin(),
        StagedGenerateFuzzySkinType::All
    );
}

#[test]
fn generate_fuzzy_painted_region_preserves_disabled_fuzzy() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut region_set = StagedGenerateRegionSet::new();

    let regions = staged_generate_fuzzy_painted_regions(
        &mut shell,
        true,
        &[parent(10, StagedGenerateFuzzySkinType::DisabledFuzzy)],
        &mut region_set,
    );

    assert_eq!(
        regions[0].derived_config().fuzzy_skin(),
        StagedGenerateFuzzySkinType::DisabledFuzzy
    );
}

#[test]
fn generate_fuzzy_painted_region_reuses_equal_derived_configs() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut region_set = StagedGenerateRegionSet::new();

    let regions = staged_generate_fuzzy_painted_regions(
        &mut shell,
        true,
        &[
            parent(10, StagedGenerateFuzzySkinType::External),
            parent(10, StagedGenerateFuzzySkinType::AllWalls),
        ],
        &mut region_set,
    );

    assert_eq!(regions[0].region_id(), 0);
    assert_eq!(regions[1].region_id(), 0);
    assert_eq!(shell.all_regions().len(), 1);
}

#[test]
fn generate_fuzzy_painted_region_reuses_equal_volume_parent_config_region() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut region_set = StagedGenerateRegionSet::new();

    let volume_regions = staged_generate_fuzzy_volume_regions(
        &mut shell,
        true,
        &[StagedGenerateFuzzyParentVolumeRegion::new(
            StagedModelVolumeType::ModelPart,
            10,
            StagedGenerateFuzzySkinType::External,
        )],
        &mut region_set,
    );
    let painted_regions = staged_generate_fuzzy_painted_regions(
        &mut shell,
        true,
        &[parent(10, StagedGenerateFuzzySkinType::AllWalls)],
        &mut region_set,
    );

    assert_eq!(volume_regions[0].region_id(), 0);
    assert_eq!(painted_regions[0].region_id(), 0);
    assert_eq!(shell.all_regions().len(), 1);
}

#[test]
fn generate_fuzzy_painted_region_empty_parents_is_noop() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut region_set = StagedGenerateRegionSet::new();

    let regions = staged_generate_fuzzy_painted_regions(&mut shell, true, &[], &mut region_set);

    assert!(regions.is_empty());
    assert!(shell.all_regions().is_empty());
}
