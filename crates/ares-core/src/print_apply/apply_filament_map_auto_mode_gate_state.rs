#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum StagedFilamentMapMode {
    AutoForFlush,
    AutoForMatch,
    Manual,
    Default,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct StagedFilamentMapAutoModeGate {
    pub(super) mode: StagedFilamentMapMode,
    pub(super) enter_auto_mode_branch: bool,
}

pub(super) fn staged_apply_filament_map_auto_mode_gate(
    mode: &str,
) -> StagedFilamentMapAutoModeGate {
    let mode = staged_filament_map_mode(mode);

    StagedFilamentMapAutoModeGate {
        mode,
        enter_auto_mode_branch: mode < StagedFilamentMapMode::Manual,
    }
}

fn staged_filament_map_mode(mode: &str) -> StagedFilamentMapMode {
    match mode {
        "fmmAutoForFlush" | "Auto For Flush" => StagedFilamentMapMode::AutoForFlush,
        "fmmAutoForMatch" | "Auto For Match" => StagedFilamentMapMode::AutoForMatch,
        "fmmManual" | "Manual" => StagedFilamentMapMode::Manual,
        "fmmDefault" => StagedFilamentMapMode::Default,
        _ => panic!("unsupported staged filament_map_mode: {mode}"),
    }
}
