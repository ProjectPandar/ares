//! Fresh-slice `PrintStateBase::Warning` data (`PrintBase.hpp:61-73,305-333`).

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WarningLevel {
    NonCritical,
    Critical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SlicingNotificationType {
    NeedSupportOn = 2,
    UnresolvedSystemPreset = 3,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrintWarning {
    pub level: WarningLevel,
    pub message_type: SlicingNotificationType,
    pub message: String,
}
