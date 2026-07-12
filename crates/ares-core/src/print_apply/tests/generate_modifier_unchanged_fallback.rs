use super::super::generate_modifier_unchanged_fallback_state::{
    StagedGenerateModifierFallbackCandidate, StagedGenerateModifierFallbackCurrent,
    staged_generate_modifier_unchanged_fallback,
};
use super::super::generate_regions_state::StagedGenerateRegionConfigKey;
use super::super::model_volume_state::StagedModelVolumeType;
use super::super::volume_cache_state::StagedExtentBox;

fn bbox(marker: f32) -> StagedExtentBox {
    StagedExtentBox::new([marker, 0.0, 0.0], [marker, 1.0, 1.0])
}

fn config(id: u64) -> StagedGenerateRegionConfigKey {
    StagedGenerateRegionConfigKey::new(id, id)
}

fn current(volume_type: StagedModelVolumeType) -> StagedGenerateModifierFallbackCurrent {
    StagedGenerateModifierFallbackCurrent::new(42, volume_type, bbox(9.0))
}

fn candidate(
    parent_region_id: usize,
    parent_volume_type: StagedModelVolumeType,
    region_id: u64,
    parent_config: u64,
    derived_config: u64,
) -> StagedGenerateModifierFallbackCandidate {
    StagedGenerateModifierFallbackCandidate::new(
        parent_region_id,
        parent_volume_type,
        region_id,
        config(parent_config),
        config(derived_config),
    )
}

#[test]
fn generate_modifier_unchanged_fallback_appends_first_unchanged_model_part() {
    let result = staged_generate_modifier_unchanged_fallback(
        current(StagedModelVolumeType::ParameterModifier),
        &[
            candidate(5, StagedModelVolumeType::ModelPart, 50, 100, 200),
            candidate(2, StagedModelVolumeType::ModelPart, 20, 110, 110),
            candidate(8, StagedModelVolumeType::ModelPart, 80, 120, 120),
        ],
        false,
    );

    assert_eq!(result.parent_model_part_id(), Some(2));
    let region = result.volume_region().expect("fallback region");
    assert_eq!(region.model_volume_id(), 42);
    assert_eq!(region.parent(), 2);
    assert_eq!(region.region_id(), 20);
    assert_eq!(region.bbox(), bbox(9.0));
}

#[test]
fn generate_modifier_unchanged_fallback_skips_modifier_parent_for_selection() {
    let result = staged_generate_modifier_unchanged_fallback(
        current(StagedModelVolumeType::ParameterModifier),
        &[
            candidate(5, StagedModelVolumeType::ParameterModifier, 50, 100, 100),
            candidate(2, StagedModelVolumeType::ModelPart, 20, 110, 110),
        ],
        false,
    );

    assert_eq!(result.parent_model_part_id(), Some(2));
    assert_eq!(result.volume_region().expect("fallback region").parent(), 2);
}

#[test]
fn generate_modifier_unchanged_fallback_skips_when_changed_append_added() {
    let result = staged_generate_modifier_unchanged_fallback(
        current(StagedModelVolumeType::ParameterModifier),
        &[candidate(2, StagedModelVolumeType::ModelPart, 20, 110, 110)],
        true,
    );

    assert_eq!(result.parent_model_part_id(), Some(2));
    assert!(result.volume_region().is_none());
}

#[test]
fn generate_modifier_unchanged_fallback_skips_without_model_part_parent() {
    let result = staged_generate_modifier_unchanged_fallback(
        current(StagedModelVolumeType::ParameterModifier),
        &[
            candidate(5, StagedModelVolumeType::ParameterModifier, 50, 100, 100),
            candidate(3, StagedModelVolumeType::ModelPart, 30, 120, 121),
        ],
        false,
    );

    assert_eq!(result.parent_model_part_id(), None);
    assert!(result.volume_region().is_none());
}

#[test]
fn generate_modifier_unchanged_fallback_non_modifier_current_is_noop() {
    let result = staged_generate_modifier_unchanged_fallback(
        current(StagedModelVolumeType::ModelPart),
        &[candidate(2, StagedModelVolumeType::ModelPart, 20, 110, 110)],
        false,
    );

    assert_eq!(result.parent_model_part_id(), None);
    assert!(result.volume_region().is_none());
}

#[test]
fn generate_modifier_unchanged_fallback_preserves_parent_region_and_bbox() {
    let result = staged_generate_modifier_unchanged_fallback(
        StagedGenerateModifierFallbackCurrent::new(
            77,
            StagedModelVolumeType::ParameterModifier,
            bbox(12.0),
        ),
        &[candidate(
            4,
            StagedModelVolumeType::ModelPart,
            400,
            100,
            100,
        )],
        false,
    );

    let region = result.volume_region().expect("fallback region");
    assert_eq!(region.model_volume_id(), 77);
    assert_eq!(region.parent(), 4);
    assert_eq!(region.region_id(), 400);
    assert_eq!(region.bbox(), bbox(12.0));
}
