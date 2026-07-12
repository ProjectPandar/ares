use std::collections::HashSet;

const COMMENT: &str = "BBS: process the filament_map related logic";
const SOURCE: &str = "print_diff";
const SET_NAME: &str = "print_diff_set";
const GUARD_KEY: &str = "filament_map_mode";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct StagedFilamentMapModeGuard {
    pub(super) comment: &'static str,
    pub(super) source: &'static str,
    pub(super) set_name: &'static str,
    pub(super) guard_key: &'static str,
    pub(super) set_size: usize,
    pub(super) enter_filament_map_processing: bool,
}

pub(super) fn staged_apply_filament_map_mode_guard(
    print_diff: &[&str],
) -> StagedFilamentMapModeGuard {
    let print_diff_set = print_diff.iter().copied().collect::<HashSet<_>>();

    StagedFilamentMapModeGuard {
        comment: COMMENT,
        source: SOURCE,
        set_name: SET_NAME,
        guard_key: GUARD_KEY,
        set_size: print_diff_set.len(),
        enter_filament_map_processing: !print_diff_set.contains(GUARD_KEY),
    }
}
