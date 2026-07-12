use std::collections::BTreeSet;

const EXTRUDER_AMS_COUNT_KEY: &str = "extruder_ams_count";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct StagedManualFilamentMapSetup {
    pub(super) entered: bool,
    pub(super) resulting_print_diff_set: Vec<&'static str>,
    pub(super) old_filament_map: Vec<i32>,
    pub(super) new_filament_map: Vec<i32>,
}

pub(super) fn staged_apply_manual_filament_map_setup(
    entered: bool,
    print_diff: &[&'static str],
    old_values: &[i32],
    new_values: &[i32],
) -> StagedManualFilamentMapSetup {
    let mut print_diff_set = print_diff.iter().copied().collect::<BTreeSet<_>>();

    if !entered {
        return StagedManualFilamentMapSetup {
            entered,
            resulting_print_diff_set: print_diff_set.into_iter().collect(),
            old_filament_map: Vec::new(),
            new_filament_map: Vec::new(),
        };
    }

    print_diff_set.remove(EXTRUDER_AMS_COUNT_KEY);

    StagedManualFilamentMapSetup {
        entered,
        resulting_print_diff_set: print_diff_set.into_iter().collect(),
        old_filament_map: old_values.to_vec(),
        new_filament_map: new_values.to_vec(),
    }
}
