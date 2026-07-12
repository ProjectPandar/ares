use std::collections::BTreeSet;

const FILAMENT_MAP_KEY: &str = "filament_map";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct StagedManualFilamentMapSameMapPrune {
    pub(super) size_matched: bool,
    pub(super) same_map: Option<bool>,
    pub(super) visited_indices: Vec<usize>,
    pub(super) first_used_difference_index: Option<usize>,
    pub(super) erased_filament_map: bool,
    pub(super) resulting_print_diff_set: Vec<&'static str>,
}

pub(super) fn staged_apply_manual_filament_map_same_map_prune(
    print_diff: &[&'static str],
    old_map: &[i32],
    new_map: &[i32],
    used_filament_indices: &[usize],
) -> StagedManualFilamentMapSameMapPrune {
    let mut print_diff_set = print_diff.iter().copied().collect::<BTreeSet<_>>();

    if old_map.len() != new_map.len() {
        return StagedManualFilamentMapSameMapPrune {
            size_matched: false,
            same_map: None,
            visited_indices: Vec::new(),
            first_used_difference_index: None,
            erased_filament_map: false,
            resulting_print_diff_set: print_diff_set.into_iter().collect(),
        };
    }

    let used_filament_set = used_filament_indices
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let mut visited_indices = Vec::new();
    let mut same_map = true;
    let mut first_used_difference_index = None;

    for index in 0..old_map.len() {
        visited_indices.push(index);
        if old_map[index] == new_map[index] || !used_filament_set.contains(&index) {
            continue;
        }
        same_map = false;
        first_used_difference_index = Some(index);
        break;
    }

    if same_map {
        print_diff_set.remove(FILAMENT_MAP_KEY);
    }

    StagedManualFilamentMapSameMapPrune {
        size_matched: true,
        same_map: Some(same_map),
        visited_indices,
        first_used_difference_index,
        erased_filament_map: same_map,
        resulting_print_diff_set: print_diff_set.into_iter().collect(),
    }
}
