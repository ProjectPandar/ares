#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum StagedSeamScarfType {
    None,
    External,
    All,
}

impl StagedSeamScarfType {
    fn is_active(self) -> bool {
        !matches!(self, Self::None)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct StagedApplyScarfJointSeamObject {
    object_seam_slope_type: StagedSeamScarfType,
    volume_seam_slope_types: Vec<Option<StagedSeamScarfType>>,
    layer_range_seam_slope_types: Vec<Option<StagedSeamScarfType>>,
}

impl StagedApplyScarfJointSeamObject {
    pub(super) fn new(
        object_seam_slope_type: StagedSeamScarfType,
        volume_seam_slope_types: Vec<Option<StagedSeamScarfType>>,
        layer_range_seam_slope_types: Vec<Option<StagedSeamScarfType>>,
    ) -> Self {
        Self {
            object_seam_slope_type,
            volume_seam_slope_types,
            layer_range_seam_slope_types,
        }
    }

    fn has_scarf_joint_seam(&self) -> bool {
        self.object_seam_slope_type.is_active()
            || self
                .volume_seam_slope_types
                .iter()
                .flatten()
                .any(|seam_slope_type| seam_slope_type.is_active())
            || self
                .layer_range_seam_slope_types
                .iter()
                .flatten()
                .any(|seam_slope_type| seam_slope_type.is_active())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedApplyScarfJointSeamSet {
    value: bool,
}

impl StagedApplyScarfJointSeamSet {
    pub(super) fn value(&self) -> bool {
        self.value
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedApplyScarfJointSeamResult {
    has_scarf_joint_seam: bool,
    config_set: Option<StagedApplyScarfJointSeamSet>,
}

impl StagedApplyScarfJointSeamResult {
    pub(super) fn has_scarf_joint_seam(&self) -> bool {
        self.has_scarf_joint_seam
    }

    pub(super) fn config_set(&self) -> Option<StagedApplyScarfJointSeamSet> {
        self.config_set
    }
}

pub(super) fn staged_apply_scarf_joint_seam(
    objects: &[StagedApplyScarfJointSeamObject],
) -> StagedApplyScarfJointSeamResult {
    let has_scarf_joint_seam = objects
        .iter()
        .any(StagedApplyScarfJointSeamObject::has_scarf_joint_seam);
    let config_set = has_scarf_joint_seam.then_some(StagedApplyScarfJointSeamSet { value: true });

    StagedApplyScarfJointSeamResult {
        has_scarf_joint_seam,
        config_set,
    }
}
