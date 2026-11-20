//! Fresh-slice `PrintStateBase::Warning` data (`PrintBase.hpp:61-73,305-333`).

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WarningLevel {
    NonCritical,
    Critical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SlicingNotificationType {
    NeedSupportOn = 2,
    EmptyGcodeLayers = 3,
    GcodeOverlap = 4,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrintWarning {
    pub level: WarningLevel,
    pub message_type: SlicingNotificationType,
    pub message: String,
}
