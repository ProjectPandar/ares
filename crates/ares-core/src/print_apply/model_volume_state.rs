#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i8)]
pub(super) enum StagedModelVolumeType {
    Invalid = -1,
    ModelPart = 0,
    NegativeVolume = 1,
    ParameterModifier = 2,
    SupportBlocker = 3,
    SupportEnforcer = 4,
}

pub(super) fn staged_model_volume_solid_or_modifier(volume_type: StagedModelVolumeType) -> bool {
    matches!(
        volume_type,
        StagedModelVolumeType::ModelPart
            | StagedModelVolumeType::NegativeVolume
            | StagedModelVolumeType::ParameterModifier
    )
}
