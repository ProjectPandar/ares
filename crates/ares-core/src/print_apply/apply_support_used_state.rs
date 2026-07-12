const ENABLE_SUPPORT_KEY: &str = "enable_support";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedApplySupportUsedAssignment {
    queried_key: &'static str,
    support_used: bool,
}

impl StagedApplySupportUsedAssignment {
    pub(super) fn queried_key(&self) -> &'static str {
        self.queried_key
    }

    pub(super) fn support_used(&self) -> bool {
        self.support_used
    }
}

pub(super) fn staged_apply_support_used(
    enable_support: Option<bool>,
) -> StagedApplySupportUsedAssignment {
    StagedApplySupportUsedAssignment {
        queried_key: ENABLE_SUPPORT_KEY,
        support_used: enable_support.unwrap_or(false),
    }
}
