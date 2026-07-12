use super::super::painted_region_state::{
    StagedPaintedRegionConfig, StagedPaintedRegionConfigChange,
    StagedPaintedRegionConfigDerivation, StagedPaintedRegionInput,
    staged_painted_region_config_apply, staged_painted_region_config_change,
    staged_painted_region_config_diff, staged_painted_region_extruder_configs,
    staged_painted_region_invalidate_event, staged_painted_region_update_gate,
};
use super::super::print_region_state::{
    StagedPrintRegionConfigKey, StagedPrintRegionRefCount, staged_print_region_ref_inc,
};
use super::super::verify_update_config_state::{
    StagedConfigValue, StagedExistingRegionConfigApply, StagedExistingRegionConfigDiff,
    StagedExistingRegionInvalidateEvent, StagedExistingRegionUpdateAction,
};

fn config(
    marker: u64,
    wall_filament: u32,
    solid_infill_filament: u32,
    sparse_infill_filament: u32,
) -> StagedPaintedRegionConfig {
    StagedPaintedRegionConfig::new(
        marker,
        wall_filament,
        solid_infill_filament,
        sparse_infill_filament,
    )
}

fn painted_region(
    painted_region_id: usize,
    parent_volume_region_id: usize,
    extruder_id: u32,
) -> StagedPaintedRegionInput {
    StagedPaintedRegionInput::new(painted_region_id, parent_volume_region_id, extruder_id)
}

fn derivation(
    painted_region_id: usize,
    parent_volume_region_id: usize,
    config: StagedPaintedRegionConfig,
) -> StagedPaintedRegionConfigDerivation {
    StagedPaintedRegionConfigDerivation::new(painted_region_id, parent_volume_region_id, config)
}

fn painted_change(
    painted_region_id: usize,
    current_config: StagedPaintedRegionConfig,
    derived_config: StagedPaintedRegionConfig,
    config_changed: bool,
) -> StagedPaintedRegionConfigChange {
    StagedPaintedRegionConfigChange::new(
        painted_region_id,
        current_config,
        derived_config,
        config_changed,
    )
}

fn config_key(fingerprint: u64) -> StagedPrintRegionConfigKey {
    StagedPrintRegionConfigKey::new(fingerprint)
}

fn value(key: &str, fingerprint: u64) -> StagedConfigValue {
    StagedConfigValue::new(key, fingerprint)
}

fn staged_diff(keys: &[&str]) -> StagedExistingRegionConfigDiff {
    StagedExistingRegionConfigDiff::new(keys.iter().map(|key| (*key).to_owned()).collect())
}

fn invalidate_event(
    current_config: StagedPrintRegionConfigKey,
    derived_config: StagedPrintRegionConfigKey,
    diff_keys: &[&str],
) -> StagedExistingRegionInvalidateEvent {
    StagedExistingRegionInvalidateEvent::new(
        current_config,
        derived_config,
        diff_keys.iter().map(|key| (*key).to_owned()).collect(),
    )
}

fn staged_apply(values: &[(&str, u64)]) -> StagedExistingRegionConfigApply {
    StagedExistingRegionConfigApply::new(
        values
            .iter()
            .map(|(key, fingerprint)| value(key, *fingerprint))
            .collect(),
        false,
        true,
    )
}

#[test]
fn painted_region_config_copies_parent_before_filament_overrides() {
    let parent_configs = [config(99, 1, 2, 3)];

    let derived =
        staged_painted_region_extruder_configs(&parent_configs, &[painted_region(4, 0, 7)]);

    assert_eq!(derived, [derivation(4, 0, config(99, 7, 7, 7))]);
}

#[test]
fn painted_region_config_sets_all_filament_roles_to_extruder_id() {
    let parent_configs = [config(11, 1, 2, 3)];

    let derived =
        staged_painted_region_extruder_configs(&parent_configs, &[painted_region(2, 0, 8)]);

    assert_eq!(derived, [derivation(2, 0, config(11, 8, 8, 8))]);
}

#[test]
fn painted_region_config_preserves_painted_region_and_parent_ids() {
    let parent_configs = [config(1, 10, 20, 30), config(2, 40, 50, 60)];

    let derived =
        staged_painted_region_extruder_configs(&parent_configs, &[painted_region(9, 1, 3)]);

    assert_eq!(derived, [derivation(9, 1, config(2, 3, 3, 3))]);
}

#[test]
fn painted_region_config_derives_multiple_regions_in_source_order_from_each_parent() {
    let parent_configs = [config(10, 1, 2, 3), config(20, 4, 5, 6)];

    let derived = staged_painted_region_extruder_configs(
        &parent_configs,
        &[
            painted_region(5, 1, 8),
            painted_region(6, 0, 7),
            painted_region(7, 1, 9),
        ],
    );

    assert_eq!(
        derived,
        [
            derivation(5, 1, config(20, 8, 8, 8)),
            derivation(6, 0, config(10, 7, 7, 7)),
            derivation(7, 1, config(20, 9, 9, 9)),
        ]
    );
}

#[test]
fn painted_region_config_does_not_mutate_parent_configs() {
    let parent_configs = [config(99, 1, 2, 3)];

    let _ = staged_painted_region_extruder_configs(&parent_configs, &[painted_region(4, 0, 7)]);

    assert_eq!(parent_configs, [config(99, 1, 2, 3)]);
}

#[test]
fn painted_region_update_gate_keeps_unchanged_zero_ref_region() {
    let current = config(99, 1, 2, 3);
    let config_change = staged_painted_region_config_change(5, current, current);
    let region = StagedPrintRegionRefCount::default();

    let action = staged_painted_region_update_gate(config_change, &region);

    assert_eq!(action, StagedExistingRegionUpdateAction::Unchanged);
}

#[test]
fn painted_region_update_gate_keeps_unchanged_referenced_region() {
    let current = config(99, 1, 2, 3);
    let config_change = staged_painted_region_config_change(5, current, current);
    let mut region = StagedPrintRegionRefCount::default();
    staged_print_region_ref_inc(&mut region);

    let action = staged_painted_region_update_gate(config_change, &region);

    assert_eq!(action, StagedExistingRegionUpdateAction::Unchanged);
}

#[test]
fn painted_region_update_gate_updates_changed_zero_ref_region() {
    let current = config(99, 1, 2, 3);
    let derived = config(99, 7, 7, 7);
    let config_change = staged_painted_region_config_change(5, current, derived);
    let region = StagedPrintRegionRefCount::default();

    let action = staged_painted_region_update_gate(config_change, &region);

    assert_eq!(action, StagedExistingRegionUpdateAction::UpdateInPlace);
}

#[test]
fn painted_region_update_gate_reslices_changed_referenced_region() {
    let current = config(99, 1, 2, 3);
    let derived = config(99, 7, 7, 7);
    let config_change = staged_painted_region_config_change(5, current, derived);
    let mut region = StagedPrintRegionRefCount::default();
    staged_print_region_ref_inc(&mut region);

    let action = staged_painted_region_update_gate(config_change, &region);

    assert_eq!(action, StagedExistingRegionUpdateAction::RequiresReslice);
}

#[test]
fn painted_region_config_change_preserves_region_and_configs() {
    let current = config(99, 1, 2, 3);
    let derived = config(99, 7, 7, 7);

    let config_change = staged_painted_region_config_change(11, current, derived);

    assert_eq!(config_change, painted_change(11, current, derived, true));
}

#[test]
fn painted_region_config_diff_preserves_current_key_order_for_update_in_place() {
    let current = [
        value("wall_filament", 1),
        value("solid_infill_filament", 2),
        value("sparse_infill_filament", 3),
    ];
    let derived = [
        value("sparse_infill_filament", 30),
        value("wall_filament", 10),
        value("solid_infill_filament", 2),
    ];

    let diff = staged_painted_region_config_diff(
        StagedExistingRegionUpdateAction::UpdateInPlace,
        &current,
        &derived,
    );

    assert_eq!(
        diff,
        staged_diff(&["wall_filament", "sparse_infill_filament"])
    );
}

#[test]
fn painted_region_config_apply_skips_unchanged_action() {
    let current_key = config_key(41);
    let derived_key = config_key(42);
    let current = [value("wall_filament", 1)];
    let derived = [value("wall_filament", 10)];
    let diff = staged_painted_region_config_diff(
        StagedExistingRegionUpdateAction::Unchanged,
        &current,
        &derived,
    );
    let event = staged_painted_region_invalidate_event(
        StagedExistingRegionUpdateAction::Unchanged,
        current_key,
        derived_key,
        &diff,
    );

    let apply = staged_painted_region_config_apply(event.as_ref(), &current, &derived, &diff);

    assert_eq!(diff, staged_diff(&[]));
    assert_eq!(event, None);
    assert_eq!(apply, None);
}

#[test]
fn painted_region_config_apply_skips_requires_reslice_action() {
    let current_key = config_key(51);
    let derived_key = config_key(52);
    let current = [value("wall_filament", 1)];
    let derived = [value("wall_filament", 10)];
    let diff = staged_painted_region_config_diff(
        StagedExistingRegionUpdateAction::RequiresReslice,
        &current,
        &derived,
    );
    let event = staged_painted_region_invalidate_event(
        StagedExistingRegionUpdateAction::RequiresReslice,
        current_key,
        derived_key,
        &diff,
    );

    let apply = staged_painted_region_config_apply(event.as_ref(), &current, &derived, &diff);

    assert_eq!(diff, staged_diff(&[]));
    assert_eq!(event, None);
    assert_eq!(apply, None);
}

#[test]
fn painted_region_invalidate_event_preserves_callback_payload_before_apply() {
    let current_key = config_key(61);
    let derived_key = config_key(62);
    let current = [value("wall_filament", 1), value("solid_infill_filament", 2)];
    let derived = [
        value("wall_filament", 10),
        value("solid_infill_filament", 20),
    ];
    let diff = staged_painted_region_config_diff(
        StagedExistingRegionUpdateAction::UpdateInPlace,
        &current,
        &derived,
    );

    let event = staged_painted_region_invalidate_event(
        StagedExistingRegionUpdateAction::UpdateInPlace,
        current_key,
        derived_key,
        &diff,
    );

    assert_eq!(
        event,
        Some(invalidate_event(
            current_key,
            derived_key,
            &["wall_filament", "solid_infill_filament"]
        ))
    );
}

#[test]
fn painted_region_config_apply_requires_invalidate_event_and_records_apply_only_state() {
    let current_key = config_key(71);
    let derived_key = config_key(72);
    let current = [value("wall_filament", 1), value("solid_infill_filament", 2)];
    let derived = [
        value("wall_filament", 10),
        value("solid_infill_filament", 20),
    ];
    let diff = staged_painted_region_config_diff(
        StagedExistingRegionUpdateAction::UpdateInPlace,
        &current,
        &derived,
    );
    let event = staged_painted_region_invalidate_event(
        StagedExistingRegionUpdateAction::UpdateInPlace,
        current_key,
        derived_key,
        &diff,
    );

    let without_event = staged_painted_region_config_apply(None, &current, &derived, &diff);
    let with_event = staged_painted_region_config_apply(event.as_ref(), &current, &derived, &diff);

    assert_eq!(without_event, None);
    assert_eq!(
        with_event,
        Some(staged_apply(&[
            ("wall_filament", 10),
            ("solid_infill_filament", 20),
        ]))
    );
}

#[test]
fn painted_region_config_apply_ignores_diff_keys_missing_from_derived() {
    let current_key = config_key(81);
    let derived_key = config_key(82);
    let current = [value("wall_filament", 1), value("missing", 5)];
    let derived = [value("wall_filament", 10)];
    let diff = staged_diff(&["wall_filament", "missing"]);
    let event = staged_painted_region_invalidate_event(
        StagedExistingRegionUpdateAction::UpdateInPlace,
        current_key,
        derived_key,
        &diff,
    );

    let apply = staged_painted_region_config_apply(event.as_ref(), &current, &derived, &diff);

    assert_eq!(
        apply,
        Some(staged_apply(&[("wall_filament", 10), ("missing", 5)]))
    );
}
