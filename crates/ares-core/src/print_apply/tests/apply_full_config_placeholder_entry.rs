use super::super::apply_full_config_placeholder_entry_state::{
    StagedApplyStatus, StagedFullConfigPlaceholderEntryEvent,
    staged_apply_full_config_placeholder_entry,
};

#[test]
fn apply_full_config_placeholder_entry_empty_diff_records_initial_state_only() {
    let entry =
        staged_apply_full_config_placeholder_entry(StagedApplyStatus::Unchanged, 3, &[], true);

    assert_eq!(entry.num_extruders, 3);
    assert!(!entry.num_extruders_changed);
    assert_eq!(entry.status, StagedApplyStatus::Unchanged);
    assert!(entry.events.is_empty());
}

#[test]
fn apply_full_config_placeholder_entry_non_empty_diff_records_log_invalidate_and_clear() {
    let entry = staged_apply_full_config_placeholder_entry(
        StagedApplyStatus::Unchanged,
        2,
        &["filament_diameter", "print_settings_id"],
        false,
    );

    assert_eq!(
        entry.events,
        [
            StagedFullConfigPlaceholderEntryEvent::LogFullConfigDiffChanged,
            StagedFullConfigPlaceholderEntryEvent::InvalidateStep {
                step: "psGCodeExport",
                invalidated: false,
            },
            StagedFullConfigPlaceholderEntryEvent::ClearPlaceholderParserConfig,
        ]
    );
}

#[test]
fn apply_full_config_placeholder_entry_false_invalidation_changes_unchanged_status() {
    let entry = staged_apply_full_config_placeholder_entry(
        StagedApplyStatus::Unchanged,
        2,
        &["filament_diameter"],
        false,
    );

    assert_eq!(entry.status, StagedApplyStatus::Changed);
}

#[test]
fn apply_full_config_placeholder_entry_true_invalidation_invalidates_status() {
    let entry = staged_apply_full_config_placeholder_entry(
        StagedApplyStatus::Changed,
        2,
        &["filament_diameter"],
        true,
    );

    assert_eq!(entry.status, StagedApplyStatus::Invalidated);
}

#[test]
fn apply_full_config_placeholder_entry_false_invalidation_does_not_downgrade_invalidated() {
    let entry = staged_apply_full_config_placeholder_entry(
        StagedApplyStatus::Invalidated,
        2,
        &["filament_diameter"],
        false,
    );

    assert_eq!(entry.status, StagedApplyStatus::Invalidated);
}

#[test]
fn apply_full_config_placeholder_entry_records_invalidate_before_clear() {
    let entry = staged_apply_full_config_placeholder_entry(
        StagedApplyStatus::Unchanged,
        2,
        &["filament_diameter"],
        false,
    );

    assert!(matches!(
        entry.events[1],
        StagedFullConfigPlaceholderEntryEvent::InvalidateStep { .. }
    ));
    assert_eq!(
        entry.events[2],
        StagedFullConfigPlaceholderEntryEvent::ClearPlaceholderParserConfig
    );
}
