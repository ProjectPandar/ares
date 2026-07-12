#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum StagedModelVolumeType {
    Invalid,
    ModelPart,
    NegativeVolume,
    ParameterModifier,
    SupportBlocker,
    SupportEnforcer,
}

pub(super) fn staged_model_volume_solid_or_modifier(volume_type: StagedModelVolumeType) -> bool {
    matches!(
        volume_type,
        StagedModelVolumeType::ModelPart
            | StagedModelVolumeType::NegativeVolume
            | StagedModelVolumeType::ParameterModifier
    )
}
