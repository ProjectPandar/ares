use std::collections::HashSet;

const GUARD_KEY: &str = "filament_map_mode";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct StagedFilamentMapModeGuard {
    pub(super) set_size: usize,
    pub(super) enter_filament_map_processing: bool,
}

pub(super) fn staged_apply_filament_map_mode_guard(
    print_diff: &[&str],
) -> StagedFilamentMapModeGuard {
    let print_diff_set = print_diff.iter().copied().collect::<HashSet<_>>();

    StagedFilamentMapModeGuard {
        set_size: print_diff_set.len(),
        enter_filament_map_processing: !print_diff_set.contains(GUARD_KEY),
    }
}
