use std::collections::BTreeSet;

const FILAMENT_MAP_KEY: &str = "filament_map";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct StagedAutoFilamentMapDiffPrune {
    pub(super) entered: bool,
    pub(super) resulting_print_diff_set: Vec<&'static str>,
    pub(super) actions: Vec<StagedAutoFilamentMapDiffPruneAction>,
    pub(super) non_actions: Vec<StagedAutoFilamentMapDiffPruneNonAction>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum StagedAutoFilamentMapDiffPruneAction {
    PrintDiffSetErase {
        key: &'static str,
    },
    OptionLookup {
        result: &'static str,
        receiver: &'static str,
        option_type: &'static str,
        key: &'static str,
        required: bool,
    },
    OptionSet {
        receiver: &'static str,
        source: &'static str,
    },
    ConfigAssignment {
        destination: &'static str,
        source: &'static str,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum StagedAutoFilamentMapDiffPruneNonAction {
    CommentedFullConfigDiffErase { key: &'static str },
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
            actions: Vec::new(),
            non_actions: Vec::new(),
        };
    }

    StagedAutoFilamentMapDiffPrune {
        entered,
        resulting_print_diff_set: print_diff_set.into_iter().collect(),
        actions: vec![
            StagedAutoFilamentMapDiffPruneAction::PrintDiffSetErase {
                key: FILAMENT_MAP_KEY,
            },
            StagedAutoFilamentMapDiffPruneAction::OptionLookup {
                result: "old_opt",
                receiver: "m_full_print_config",
                option_type: "ConfigOptionInts",
                key: FILAMENT_MAP_KEY,
                required: true,
            },
            StagedAutoFilamentMapDiffPruneAction::OptionLookup {
                result: "new_opt",
                receiver: "new_full_config",
                option_type: "ConfigOptionInts",
                key: FILAMENT_MAP_KEY,
                required: true,
            },
            StagedAutoFilamentMapDiffPruneAction::OptionSet {
                receiver: "old_opt",
                source: "new_opt",
            },
            StagedAutoFilamentMapDiffPruneAction::ConfigAssignment {
                destination: "m_config.filament_map",
                source: "*new_opt",
            },
        ],
        non_actions: vec![
            StagedAutoFilamentMapDiffPruneNonAction::CommentedFullConfigDiffErase {
                key: FILAMENT_MAP_KEY,
            },
        ],
    }
}
