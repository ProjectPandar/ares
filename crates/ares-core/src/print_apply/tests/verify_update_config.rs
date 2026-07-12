use super::super::print_region_state::{
    StagedPrintRegionConfigKey, StagedPrintRegionRefCount, staged_print_region_ref_cnt,
    staged_print_region_ref_inc,
};
use super::super::verify_update_config_state::{
    StagedConfigValue, StagedExistingRegionConfigApply, StagedExistingRegionConfigChange,
    StagedExistingRegionRefIncrement, StagedExistingRegionUpdateAction,
    StagedMissingOverrideConfigGate, staged_verify_update_existing_region_config_change,
    staged_verify_update_existing_region_ref_inc, staged_verify_update_existing_region_update_gate,
    staged_verify_update_missing_override_config_gate,
};

fn config_key(fingerprint: u64) -> StagedPrintRegionConfigKey {
    StagedPrintRegionConfigKey::new(fingerprint)
}

fn missing_override_gate(
    parent_region_id: usize,
    parent_config: StagedPrintRegionConfigKey,
    derived_config: StagedPrintRegionConfigKey,
    requires_reslice: bool,
) -> StagedMissingOverrideConfigGate {
    StagedMissingOverrideConfigGate::new(
        parent_region_id,
        parent_config,
        derived_config,
        requires_reslice,
    )
}

#[test]
fn verify_update_missing_override_config_gate_keeps_equal_configs_reusable() {
    let parent_config = config_key(7);
    let derived_config = config_key(7);

    let gate = staged_verify_update_missing_override_config_gate(3, parent_config, derived_config);

    assert_eq!(
        gate,
        missing_override_gate(3, parent_config, derived_config, false)
    );
}

#[test]
fn verify_update_missing_override_config_gate_requires_reslice_for_different_configs() {
    let parent_config = config_key(7);
    let derived_config = config_key(9);

    let gate = staged_verify_update_missing_override_config_gate(3, parent_config, derived_config);

    assert_eq!(
        gate,
        missing_override_gate(3, parent_config, derived_config, true)
    );
}

#[test]
fn verify_update_missing_override_config_gate_preserves_parent_and_config_values() {
    let parent_config = config_key(11);
    let derived_config = config_key(12);

    let gate = staged_verify_update_missing_override_config_gate(5, parent_config, derived_config);

    assert_eq!(
        gate,
        missing_override_gate(5, parent_config, derived_config, true)
    );
}

#[test]
fn verify_update_missing_override_config_gate_parent_id_does_not_affect_equality() {
    let parent_config = config_key(13);
    let derived_config = config_key(13);

    let first = staged_verify_update_missing_override_config_gate(1, parent_config, derived_config);
    let second =
        staged_verify_update_missing_override_config_gate(2, parent_config, derived_config);

    assert_eq!(
        first,
        missing_override_gate(1, parent_config, derived_config, false)
    );
    assert_eq!(
        second,
        missing_override_gate(2, parent_config, derived_config, false)
    );
}

fn existing_region_config_change(
    volume_region_id: usize,
    current_config: StagedPrintRegionConfigKey,
    derived_config: StagedPrintRegionConfigKey,
    config_changed: bool,
) -> StagedExistingRegionConfigChange {
    StagedExistingRegionConfigChange::new(
        volume_region_id,
        current_config,
        derived_config,
        config_changed,
    )
}

#[test]
fn verify_update_existing_region_config_change_keeps_equal_configs_unchanged() {
    let current_config = config_key(21);
    let derived_config = config_key(21);

    let change =
        staged_verify_update_existing_region_config_change(4, current_config, derived_config);

    assert_eq!(
        change,
        existing_region_config_change(4, current_config, derived_config, false)
    );
}

#[test]
fn verify_update_existing_region_config_change_marks_different_configs_changed() {
    let current_config = config_key(21);
    let derived_config = config_key(22);

    let change =
        staged_verify_update_existing_region_config_change(4, current_config, derived_config);

    assert_eq!(
        change,
        existing_region_config_change(4, current_config, derived_config, true)
    );
}

#[test]
fn verify_update_existing_region_config_change_preserves_region_and_config_values() {
    let current_config = config_key(31);
    let derived_config = config_key(32);

    let change =
        staged_verify_update_existing_region_config_change(8, current_config, derived_config);

    assert_eq!(
        change,
        existing_region_config_change(8, current_config, derived_config, true)
    );
}

#[test]
fn verify_update_existing_region_config_change_region_id_does_not_affect_equality() {
    let current_config = config_key(41);
    let derived_config = config_key(41);

    let first =
        staged_verify_update_existing_region_config_change(1, current_config, derived_config);
    let second =
        staged_verify_update_existing_region_config_change(2, current_config, derived_config);

    assert_eq!(
        first,
        existing_region_config_change(1, current_config, derived_config, false)
    );
    assert_eq!(
        second,
        existing_region_config_change(2, current_config, derived_config, false)
    );
}

fn existing_region_change(
    current_config: StagedPrintRegionConfigKey,
    derived_config: StagedPrintRegionConfigKey,
) -> StagedExistingRegionConfigChange {
    staged_verify_update_existing_region_config_change(4, current_config, derived_config)
}

#[test]
fn verify_update_existing_region_update_gate_keeps_unchanged_zero_ref_region() {
    let region = StagedPrintRegionRefCount::default();
    let change = existing_region_change(config_key(51), config_key(51));

    let action = staged_verify_update_existing_region_update_gate(change, &region);

    assert_eq!(action, StagedExistingRegionUpdateAction::Unchanged);
}

#[test]
fn verify_update_existing_region_update_gate_updates_changed_zero_ref_region() {
    let region = StagedPrintRegionRefCount::default();
    let change = existing_region_change(config_key(51), config_key(52));

    let action = staged_verify_update_existing_region_update_gate(change, &region);

    assert_eq!(action, StagedExistingRegionUpdateAction::UpdateInPlace);
}

#[test]
fn verify_update_existing_region_update_gate_reslices_changed_referenced_region() {
    let mut region = StagedPrintRegionRefCount::default();
    staged_print_region_ref_inc(&mut region);
    let change = existing_region_change(config_key(51), config_key(52));

    let action = staged_verify_update_existing_region_update_gate(change, &region);

    assert_eq!(action, StagedExistingRegionUpdateAction::RequiresReslice);
}

#[test]
fn verify_update_existing_region_update_gate_keeps_unchanged_referenced_region() {
    let mut region = StagedPrintRegionRefCount::default();
    staged_print_region_ref_inc(&mut region);
    let change = existing_region_change(config_key(51), config_key(51));

    let action = staged_verify_update_existing_region_update_gate(change, &region);

    assert_eq!(action, StagedExistingRegionUpdateAction::Unchanged);
}

fn existing_region_ref_increment(count_after: i32) -> StagedExistingRegionRefIncrement {
    StagedExistingRegionRefIncrement::new(count_after)
}

fn config_apply() -> StagedExistingRegionConfigApply {
    StagedExistingRegionConfigApply::new(vec![StagedConfigValue::new("wall_loops", 7)], false, true)
}

#[test]
fn verify_update_existing_region_ref_inc_increments_unchanged_zero_ref_region() {
    let mut region = StagedPrintRegionRefCount::default();

    let increment = staged_verify_update_existing_region_ref_inc(
        StagedExistingRegionUpdateAction::Unchanged,
        None,
        &mut region,
    );

    assert_eq!(increment, Some(existing_region_ref_increment(1)));
    assert_eq!(staged_print_region_ref_cnt(&region), 1);
}

#[test]
fn verify_update_existing_region_ref_inc_accumulates_unchanged_referenced_region() {
    let mut region = StagedPrintRegionRefCount::default();
    staged_print_region_ref_inc(&mut region);
    staged_print_region_ref_inc(&mut region);

    let increment = staged_verify_update_existing_region_ref_inc(
        StagedExistingRegionUpdateAction::Unchanged,
        None,
        &mut region,
    );

    assert_eq!(increment, Some(existing_region_ref_increment(3)));
    assert_eq!(staged_print_region_ref_cnt(&region), 3);
}

#[test]
fn verify_update_existing_region_ref_inc_updates_in_place_when_apply_exists() {
    let mut region = StagedPrintRegionRefCount::default();
    let apply = config_apply();

    let increment = staged_verify_update_existing_region_ref_inc(
        StagedExistingRegionUpdateAction::UpdateInPlace,
        Some(&apply),
        &mut region,
    );

    assert_eq!(increment, Some(existing_region_ref_increment(1)));
    assert_eq!(staged_print_region_ref_cnt(&region), 1);
}

#[test]
fn verify_update_existing_region_ref_inc_skips_update_in_place_without_apply() {
    let mut region = StagedPrintRegionRefCount::default();

    let increment = staged_verify_update_existing_region_ref_inc(
        StagedExistingRegionUpdateAction::UpdateInPlace,
        None,
        &mut region,
    );

    assert_eq!(increment, None);
    assert_eq!(staged_print_region_ref_cnt(&region), 0);
}

#[test]
fn verify_update_existing_region_ref_inc_skips_requires_reslice_even_with_apply() {
    let mut region = StagedPrintRegionRefCount::default();
    let apply = config_apply();

    let increment = staged_verify_update_existing_region_ref_inc(
        StagedExistingRegionUpdateAction::RequiresReslice,
        Some(&apply),
        &mut region,
    );

    assert_eq!(increment, None);
    assert_eq!(staged_print_region_ref_cnt(&region), 0);
}
