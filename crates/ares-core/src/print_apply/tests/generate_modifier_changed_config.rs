use super::super::generate_modifier_changed_config_state::{
    StagedGenerateModifierChangedCandidate, StagedGenerateModifierCurrent,
    staged_generate_modifier_changed_config_regions,
};
use super::super::generate_regions_state::{
    StagedGeneratePrintObjectRegions, StagedGenerateRegionConfigKey, StagedGenerateRegionSet,
};
use super::super::model_volume_state::StagedModelVolumeType;
use super::super::volume_cache_state::StagedExtentBox;

fn bbox(marker: f32) -> StagedExtentBox {
    StagedExtentBox::new([marker, 0.0, 0.0], [marker, 1.0, 1.0])
}

fn config(id: u64) -> StagedGenerateRegionConfigKey {
    StagedGenerateRegionConfigKey::new(id, id)
}

fn current(volume_type: StagedModelVolumeType) -> StagedGenerateModifierCurrent {
    StagedGenerateModifierCurrent::new(42, volume_type, bbox(9.0))
}

fn candidate(
    parent_region_id: usize,
    parent_config: u64,
    derived_config: u64,
) -> StagedGenerateModifierChangedCandidate {
    StagedGenerateModifierChangedCandidate::new(
        parent_region_id,
        config(parent_config),
        config(derived_config),
    )
}

#[test]
fn generate_modifier_changed_config_appends_changed_candidate() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut region_set = StagedGenerateRegionSet::new();

    let result = staged_generate_modifier_changed_config_regions(
        &mut shell,
        current(StagedModelVolumeType::ParameterModifier),
        &[candidate(3, 100, 200)],
        &mut region_set,
    );

    assert!(result.added());
    assert_eq!(result.volume_regions().len(), 1);
    assert_eq!(result.volume_regions()[0].model_volume_id(), 42);
    assert_eq!(result.volume_regions()[0].parent(), 3);
    assert_eq!(result.volume_regions()[0].region_id(), 0);
    assert_eq!(result.volume_regions()[0].bbox(), bbox(9.0));
    assert_eq!(shell.all_regions().len(), 1);
}

#[test]
fn generate_modifier_changed_config_skips_unchanged_candidate() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut region_set = StagedGenerateRegionSet::new();

    let result = staged_generate_modifier_changed_config_regions(
        &mut shell,
        current(StagedModelVolumeType::ParameterModifier),
        &[candidate(3, 100, 100)],
        &mut region_set,
    );

    assert!(!result.added());
    assert!(result.volume_regions().is_empty());
    assert!(shell.all_regions().is_empty());
}

#[test]
fn generate_modifier_changed_config_preserves_candidate_order() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut region_set = StagedGenerateRegionSet::new();

    let result = staged_generate_modifier_changed_config_regions(
        &mut shell,
        current(StagedModelVolumeType::ParameterModifier),
        &[
            candidate(5, 100, 200),
            candidate(2, 110, 210),
            candidate(8, 120, 220),
        ],
        &mut region_set,
    );

    let parents = result
        .volume_regions()
        .iter()
        .map(|region| region.parent())
        .collect::<Vec<_>>();
    assert_eq!(parents, vec![5, 2, 8]);
}

#[test]
fn generate_modifier_changed_config_reuses_equal_derived_configs() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut region_set = StagedGenerateRegionSet::new();

    let result = staged_generate_modifier_changed_config_regions(
        &mut shell,
        current(StagedModelVolumeType::ParameterModifier),
        &[candidate(5, 100, 200), candidate(2, 110, 200)],
        &mut region_set,
    );

    assert_eq!(result.volume_regions()[0].region_id(), 0);
    assert_eq!(result.volume_regions()[1].region_id(), 0);
    assert_eq!(shell.all_regions().len(), 1);
}

#[test]
fn generate_modifier_changed_config_creates_distinct_regions_for_distinct_configs() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut region_set = StagedGenerateRegionSet::new();

    let result = staged_generate_modifier_changed_config_regions(
        &mut shell,
        current(StagedModelVolumeType::ParameterModifier),
        &[candidate(5, 100, 200), candidate(2, 110, 210)],
        &mut region_set,
    );

    assert_eq!(result.volume_regions()[0].region_id(), 0);
    assert_eq!(result.volume_regions()[1].region_id(), 1);
    assert_eq!(shell.all_regions().len(), 2);
}

#[test]
fn generate_modifier_changed_config_non_modifier_current_is_noop() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut region_set = StagedGenerateRegionSet::new();

    let result = staged_generate_modifier_changed_config_regions(
        &mut shell,
        current(StagedModelVolumeType::ModelPart),
        &[candidate(3, 100, 200)],
        &mut region_set,
    );

    assert!(!result.added());
    assert!(result.volume_regions().is_empty());
    assert!(shell.all_regions().is_empty());
}

#[test]
fn generate_modifier_changed_config_preserves_parent_and_bbox_identity() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut region_set = StagedGenerateRegionSet::new();

    let result = staged_generate_modifier_changed_config_regions(
        &mut shell,
        StagedGenerateModifierCurrent::new(
            77,
            StagedModelVolumeType::ParameterModifier,
            bbox(12.0),
        ),
        &[candidate(4, 100, 200)],
        &mut region_set,
    );

    let region = &result.volume_regions()[0];
    assert_eq!(region.model_volume_id(), 77);
    assert_eq!(region.parent(), 4);
    assert_eq!(region.region_id(), 0);
    assert_eq!(region.bbox(), bbox(12.0));
}
