#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedApplySupportUsedAssignment {
    support_used: bool,
}

impl StagedApplySupportUsedAssignment {
    pub(super) fn support_used(&self) -> bool {
        self.support_used
    }
}

pub(super) fn staged_apply_support_used(
    enable_support: Option<bool>,
) -> StagedApplySupportUsedAssignment {
    StagedApplySupportUsedAssignment {
        support_used: enable_support.unwrap_or(false),
    }
}
