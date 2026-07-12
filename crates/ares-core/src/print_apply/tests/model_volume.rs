use super::super::model_volume_state::{
    StagedModelVolumeType, staged_model_volume_solid_or_modifier,
};

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
