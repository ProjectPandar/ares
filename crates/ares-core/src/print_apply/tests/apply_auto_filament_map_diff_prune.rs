use super::super::apply_auto_filament_map_diff_prune_state::{
    StagedAutoFilamentMapDiffPruneAction, StagedAutoFilamentMapDiffPruneNonAction,
    staged_apply_auto_filament_map_diff_prune,
};

#[test]
fn apply_auto_filament_map_diff_prune_skips_when_filament_map_absent() {
    let state = staged_apply_auto_filament_map_diff_prune(&["layer_height", "infill_density"]);

    assert!(!state.entered);
    assert_eq!(
        state.resulting_print_diff_set,
        ["infill_density", "layer_height"]
    );
    assert!(state.actions.is_empty());
    assert!(state.non_actions.is_empty());
}

#[test]
fn apply_auto_filament_map_diff_prune_enters_and_erases_filament_map() {
    let state = staged_apply_auto_filament_map_diff_prune(&["filament_map", "layer_height"]);

    assert!(state.entered);
    assert_eq!(state.resulting_print_diff_set, ["layer_height"]);
    assert_eq!(
        state.actions[0],
        StagedAutoFilamentMapDiffPruneAction::PrintDiffSetErase {
            key: "filament_map"
        }
    );
}

#[test]
fn apply_auto_filament_map_diff_prune_suppresses_duplicate_membership() {
    let state = staged_apply_auto_filament_map_diff_prune(&[
        "filament_map",
        "layer_height",
        "filament_map",
        "layer_height",
    ]);

    assert!(state.entered);
    assert_eq!(state.resulting_print_diff_set, ["layer_height"]);
    assert_eq!(
        state
            .actions
            .iter()
            .filter(|action| matches!(
                action,
                StagedAutoFilamentMapDiffPruneAction::PrintDiffSetErase {
                    key: "filament_map"
                }
            ))
            .count(),
        1
    );
}

#[test]
fn apply_auto_filament_map_diff_prune_records_required_option_lookups() {
    let state = staged_apply_auto_filament_map_diff_prune(&["filament_map"]);

    assert_eq!(
        state.actions[1],
        StagedAutoFilamentMapDiffPruneAction::OptionLookup {
            result: "old_opt",
            receiver: "m_full_print_config",
            option_type: "ConfigOptionInts",
            key: "filament_map",
            required: true,
        }
    );
    assert_eq!(
        state.actions[2],
        StagedAutoFilamentMapDiffPruneAction::OptionLookup {
            result: "new_opt",
            receiver: "new_full_config",
            option_type: "ConfigOptionInts",
            key: "filament_map",
            required: true,
        }
    );
}

#[test]
fn apply_auto_filament_map_diff_prune_records_set_and_assignment_actions() {
    let state = staged_apply_auto_filament_map_diff_prune(&["filament_map"]);

    assert_eq!(
        state.actions[3],
        StagedAutoFilamentMapDiffPruneAction::OptionSet {
            receiver: "old_opt",
            source: "new_opt",
        }
    );
    assert_eq!(
        state.actions[4],
        StagedAutoFilamentMapDiffPruneAction::ConfigAssignment {
            destination: "m_config.filament_map",
            source: "*new_opt",
        }
    );
}

#[test]
fn apply_auto_filament_map_diff_prune_records_commented_full_config_erase_as_non_action() {
    let state = staged_apply_auto_filament_map_diff_prune(&["filament_map"]);

    assert_eq!(
        state.non_actions,
        [
            StagedAutoFilamentMapDiffPruneNonAction::CommentedFullConfigDiffErase {
                key: "filament_map"
            }
        ]
    );
}
