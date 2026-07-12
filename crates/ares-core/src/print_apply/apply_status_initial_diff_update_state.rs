#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum StagedApplyStatus {
    Unchanged,
    Changed,
    Invalidated,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct StagedInitialDiffStatusLog {
    pub(super) print_diff_len: usize,
    pub(super) object_diff_len: usize,
    pub(super) region_diff_len: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct StagedInitialDiffStatusUpdate {
    pub(super) status: StagedApplyStatus,
    pub(super) log: Option<StagedInitialDiffStatusLog>,
}

pub(super) fn staged_update_apply_status(status: &mut StagedApplyStatus, invalidated: bool) {
    let next = if invalidated {
        StagedApplyStatus::Invalidated
    } else {
        StagedApplyStatus::Changed
    };
    *status = (*status).max(next);
}

pub(super) fn staged_apply_status_initial_diff_update(
    print_diff_len: usize,
    object_diff_len: usize,
    region_diff_len: usize,
) -> StagedInitialDiffStatusUpdate {
    let mut status = StagedApplyStatus::Unchanged;
    let has_diff = print_diff_len != 0 || object_diff_len != 0 || region_diff_len != 0;
    let log = if has_diff {
        staged_update_apply_status(&mut status, false);
        Some(StagedInitialDiffStatusLog {
            print_diff_len,
            object_diff_len,
            region_diff_len,
        })
    } else {
        None
    };

    StagedInitialDiffStatusUpdate { status, log }
}
