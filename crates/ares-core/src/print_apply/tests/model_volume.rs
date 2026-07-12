use super::super::model_volume_state::{
    StagedModelVolumeType, staged_model_volume_solid_or_modifier,
};

#[test]
fn model_volume_type_variants_preserve_upstream_source_order() {
    assert_eq!(StagedModelVolumeType::Invalid as i8, -1);
    assert_eq!(StagedModelVolumeType::ModelPart as i8, 0);
    assert_eq!(StagedModelVolumeType::NegativeVolume as i8, 1);
    assert_eq!(StagedModelVolumeType::ParameterModifier as i8, 2);
    assert_eq!(StagedModelVolumeType::SupportBlocker as i8, 3);
    assert_eq!(StagedModelVolumeType::SupportEnforcer as i8, 4);
}

#[test]
fn model_volume_solid_or_modifier_accepts_solid_modifier_variants() {
    for volume_type in [
        StagedModelVolumeType::ModelPart,
        StagedModelVolumeType::NegativeVolume,
        StagedModelVolumeType::ParameterModifier,
    ] {
        assert!(staged_model_volume_solid_or_modifier(volume_type));
    }
}

#[test]
fn model_volume_solid_or_modifier_rejects_invalid_and_support_variants() {
    for volume_type in [
        StagedModelVolumeType::Invalid,
        StagedModelVolumeType::SupportBlocker,
        StagedModelVolumeType::SupportEnforcer,
    ] {
        assert!(!staged_model_volume_solid_or_modifier(volume_type));
    }
}
