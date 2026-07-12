#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub(super) enum StagedApplyStatus {
    Unchanged = 0,
    Changed = 1,
    Invalidated = 2,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum StagedPrintDiffConfigInvalidationEvent {
    LockStateMutex,
    InvalidateStateByConfigOptions {
        receiver: &'static str,
        config_source: &'static str,
        print_diff: Vec<&'static str>,
        invalidated: bool,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct StagedPrintDiffConfigInvalidation {
    pub(super) status: StagedApplyStatus,
    pub(super) events: Vec<StagedPrintDiffConfigInvalidationEvent>,
}

fn staged_update_apply_status(status: &mut StagedApplyStatus, invalidated: bool) {
    let next = if invalidated {
        StagedApplyStatus::Invalidated
    } else {
        StagedApplyStatus::Changed
    };
    *status = (*status).max(next);
}

pub(super) fn staged_apply_print_diff_config_invalidation(
    prior_status: StagedApplyStatus,
    print_diff: &[&'static str],
    invalidation_result: bool,
) -> StagedPrintDiffConfigInvalidation {
    let mut status = prior_status;
    let mut events = vec![StagedPrintDiffConfigInvalidationEvent::LockStateMutex];

    if !print_diff.is_empty() {
        events.push(
            StagedPrintDiffConfigInvalidationEvent::InvalidateStateByConfigOptions {
                receiver: "this",
                config_source: "new_full_config",
                print_diff: print_diff.to_vec(),
                invalidated: invalidation_result,
            },
        );
        staged_update_apply_status(&mut status, invalidation_result);
    }

    StagedPrintDiffConfigInvalidation { status, events }
}
