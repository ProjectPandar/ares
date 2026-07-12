use std::collections::BTreeSet;

const FILAMENT_MAP_KEY: &str = "filament_map";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct StagedAutoFilamentMapDiffPrune {
    pub(super) entered: bool,
    pub(super) resulting_print_diff_set: Vec<&'static str>,
}

pub(super) fn staged_apply_auto_filament_map_diff_prune(
    print_diff: &[&'static str],
) -> StagedAutoFilamentMapDiffPrune {
    let mut print_diff_set = print_diff.iter().copied().collect::<BTreeSet<_>>();
    let entered = print_diff_set.remove(FILAMENT_MAP_KEY);

    if !entered {
        return StagedAutoFilamentMapDiffPrune {
            entered,
            resulting_print_diff_set: print_diff_set.into_iter().collect(),
        };
    }

    StagedAutoFilamentMapDiffPrune {
        entered,
        resulting_print_diff_set: print_diff_set.into_iter().collect(),
    }
}
