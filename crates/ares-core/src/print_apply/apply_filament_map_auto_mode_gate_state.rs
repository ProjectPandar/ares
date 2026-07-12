const SOURCE_CONFIG: &str = "new_full_config";
const OPTION_KEY: &str = "filament_map_mode";
const VALUE_NAME: &str = "map_mode";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum StagedFilamentMapMode {
    AutoForFlush,
    AutoForMatch,
    Manual,
    Default,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct StagedFilamentMapAutoModeGate {
    pub(super) source_config: &'static str,
    pub(super) option_key: &'static str,
    pub(super) required: bool,
    pub(super) value_name: &'static str,
    pub(super) mode: StagedFilamentMapMode,
    pub(super) enter_auto_mode_branch: bool,
}

pub(super) fn staged_apply_filament_map_auto_mode_gate(
    mode: &str,
) -> StagedFilamentMapAutoModeGate {
    let mode = staged_filament_map_mode(mode);

    StagedFilamentMapAutoModeGate {
        source_config: SOURCE_CONFIG,
        option_key: OPTION_KEY,
        required: true,
        value_name: VALUE_NAME,
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
