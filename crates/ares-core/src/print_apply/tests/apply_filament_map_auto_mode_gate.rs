use super::super::apply_filament_map_auto_mode_gate_state::{
    StagedFilamentMapMode, staged_apply_filament_map_auto_mode_gate,
};

#[test]
fn apply_filament_map_auto_mode_gate_accepts_auto_for_flush_forms() {
    let internal = staged_apply_filament_map_auto_mode_gate("fmmAutoForFlush");
    let display = staged_apply_filament_map_auto_mode_gate("Auto For Flush");

    assert_eq!(internal.mode, StagedFilamentMapMode::AutoForFlush);
    assert_eq!(display.mode, StagedFilamentMapMode::AutoForFlush);
    assert!(internal.enter_auto_mode_branch);
    assert!(display.enter_auto_mode_branch);
}

#[test]
fn apply_filament_map_auto_mode_gate_accepts_auto_for_match_forms() {
    let internal = staged_apply_filament_map_auto_mode_gate("fmmAutoForMatch");
    let display = staged_apply_filament_map_auto_mode_gate("Auto For Match");

    assert_eq!(internal.mode, StagedFilamentMapMode::AutoForMatch);
    assert_eq!(display.mode, StagedFilamentMapMode::AutoForMatch);
    assert!(internal.enter_auto_mode_branch);
    assert!(display.enter_auto_mode_branch);
}

#[test]
fn apply_filament_map_auto_mode_gate_rejects_manual_forms() {
    let internal = staged_apply_filament_map_auto_mode_gate("fmmManual");
    let display = staged_apply_filament_map_auto_mode_gate("Manual");

    assert_eq!(internal.mode, StagedFilamentMapMode::Manual);
    assert_eq!(display.mode, StagedFilamentMapMode::Manual);
    assert!(!internal.enter_auto_mode_branch);
    assert!(!display.enter_auto_mode_branch);
}

#[test]
fn apply_filament_map_auto_mode_gate_rejects_default_variant() {
    let gate = staged_apply_filament_map_auto_mode_gate("fmmDefault");

    assert_eq!(gate.mode, StagedFilamentMapMode::Default);
    assert!(!gate.enter_auto_mode_branch);
}
