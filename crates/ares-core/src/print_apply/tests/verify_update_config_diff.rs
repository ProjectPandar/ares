use super::super::print_region_state::StagedPrintRegionConfigKey;
use super::super::verify_update_config_state::{
    StagedConfigValue, StagedExistingRegionConfigApply, StagedExistingRegionConfigDiff,
    StagedExistingRegionInvalidateEvent, StagedExistingRegionUpdateAction,
    staged_verify_update_existing_region_config_apply,
    staged_verify_update_existing_region_config_diff,
    staged_verify_update_existing_region_invalidate_event,
};

fn config_key(fingerprint: u64) -> StagedPrintRegionConfigKey {
    StagedPrintRegionConfigKey::new(fingerprint)
}

fn staged_value(key: &str, fingerprint: u64) -> StagedConfigValue {
    StagedConfigValue::new(key, fingerprint)
}

fn staged_diff(keys: &[&str]) -> StagedExistingRegionConfigDiff {
    StagedExistingRegionConfigDiff::new(keys.iter().map(|key| (*key).to_owned()).collect())
}

#[test]
fn verify_update_existing_region_config_diff_preserves_current_key_order() {
    let current = [
        staged_value("wall_loops", 1),
        staged_value("sparse_infill_density", 2),
        staged_value("top_shell_layers", 3),
    ];
    let derived = [
        staged_value("top_shell_layers", 30),
        staged_value("wall_loops", 10),
        staged_value("sparse_infill_density", 2),
    ];

    let diff = staged_verify_update_existing_region_config_diff(
        StagedExistingRegionUpdateAction::UpdateInPlace,
        &current,
        &derived,
    );

    assert_eq!(diff, staged_diff(&["wall_loops", "top_shell_layers"]));
}

#[test]
fn verify_update_existing_region_config_diff_ignores_keys_missing_from_derived() {
    let current = [staged_value("wall_loops", 1), staged_value("missing", 2)];
    let derived = [staged_value("wall_loops", 10)];

    let diff = staged_verify_update_existing_region_config_diff(
        StagedExistingRegionUpdateAction::UpdateInPlace,
        &current,
        &derived,
    );

    assert_eq!(diff, staged_diff(&["wall_loops"]));
}

#[test]
fn verify_update_existing_region_config_diff_ignores_keys_missing_from_current() {
    let current = [staged_value("wall_loops", 1)];
    let derived = [
        staged_value("wall_loops", 1),
        staged_value("derived_only", 9),
    ];

    let diff = staged_verify_update_existing_region_config_diff(
        StagedExistingRegionUpdateAction::UpdateInPlace,
        &current,
        &derived,
    );

    assert_eq!(diff, staged_diff(&[]));
}

#[test]
fn verify_update_existing_region_config_diff_suppresses_equal_values() {
    let current = [
        staged_value("wall_loops", 1),
        staged_value("top_shell_layers", 3),
    ];
    let derived = [
        staged_value("wall_loops", 1),
        staged_value("top_shell_layers", 30),
    ];

    let diff = staged_verify_update_existing_region_config_diff(
        StagedExistingRegionUpdateAction::UpdateInPlace,
        &current,
        &derived,
    );

    assert_eq!(diff, staged_diff(&["top_shell_layers"]));
}

#[test]
fn verify_update_existing_region_config_diff_preserves_duplicate_current_keys() {
    let current = [
        staged_value("wall_loops", 1),
        staged_value("top_shell_layers", 3),
        staged_value("wall_loops", 1),
    ];
    let derived = [
        staged_value("wall_loops", 10),
        staged_value("top_shell_layers", 30),
    ];

    let diff = staged_verify_update_existing_region_config_diff(
        StagedExistingRegionUpdateAction::UpdateInPlace,
        &current,
        &derived,
    );

    assert_eq!(
        diff,
        staged_diff(&["wall_loops", "top_shell_layers", "wall_loops"])
    );
}

#[test]
fn verify_update_existing_region_config_diff_skips_unchanged_action() {
    let current = [staged_value("wall_loops", 1)];
    let derived = [staged_value("wall_loops", 10)];

    let diff = staged_verify_update_existing_region_config_diff(
        StagedExistingRegionUpdateAction::Unchanged,
        &current,
        &derived,
    );

    assert_eq!(diff, staged_diff(&[]));
}

#[test]
fn verify_update_existing_region_config_diff_skips_requires_reslice_action() {
    let current = [staged_value("wall_loops", 1)];
    let derived = [staged_value("wall_loops", 10)];

    let diff = staged_verify_update_existing_region_config_diff(
        StagedExistingRegionUpdateAction::RequiresReslice,
        &current,
        &derived,
    );

    assert_eq!(diff, staged_diff(&[]));
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

#[test]
fn verify_update_existing_region_invalidate_event_preserves_config_argument_order() {
    let current_config = config_key(61);
    let derived_config = config_key(62);
    let diff = staged_diff(&["wall_loops"]);

    let event = staged_verify_update_existing_region_invalidate_event(
        StagedExistingRegionUpdateAction::UpdateInPlace,
        current_config,
        derived_config,
        &diff,
    );

    assert_eq!(
        event,
        Some(invalidate_event(
            current_config,
            derived_config,
            &["wall_loops"]
        ))
    );
}

#[test]
fn verify_update_existing_region_invalidate_event_preserves_diff_key_order() {
    let current_config = config_key(61);
    let derived_config = config_key(62);
    let diff = staged_diff(&["wall_loops", "top_shell_layers", "sparse_infill_density"]);

    let event = staged_verify_update_existing_region_invalidate_event(
        StagedExistingRegionUpdateAction::UpdateInPlace,
        current_config,
        derived_config,
        &diff,
    );

    assert_eq!(
        event,
        Some(invalidate_event(
            current_config,
            derived_config,
            &["wall_loops", "top_shell_layers", "sparse_infill_density"]
        ))
    );
}

#[test]
fn verify_update_existing_region_invalidate_event_emits_empty_diff_update() {
    let current_config = config_key(61);
    let derived_config = config_key(62);
    let diff = staged_diff(&[]);

    let event = staged_verify_update_existing_region_invalidate_event(
        StagedExistingRegionUpdateAction::UpdateInPlace,
        current_config,
        derived_config,
        &diff,
    );

    assert_eq!(
        event,
        Some(invalidate_event(current_config, derived_config, &[]))
    );
}

#[test]
fn verify_update_existing_region_invalidate_event_skips_unchanged_action() {
    let diff = staged_diff(&["wall_loops"]);

    let event = staged_verify_update_existing_region_invalidate_event(
        StagedExistingRegionUpdateAction::Unchanged,
        config_key(61),
        config_key(62),
        &diff,
    );

    assert_eq!(event, None);
}

#[test]
fn verify_update_existing_region_invalidate_event_skips_requires_reslice_action() {
    let diff = staged_diff(&["wall_loops"]);

    let event = staged_verify_update_existing_region_invalidate_event(
        StagedExistingRegionUpdateAction::RequiresReslice,
        config_key(61),
        config_key(62),
        &diff,
    );

    assert_eq!(event, None);
}

fn staged_apply(
    values: &[(&str, u64)],
    ignore_nonexistent: bool,
    hash_refreshed: bool,
) -> StagedExistingRegionConfigApply {
    StagedExistingRegionConfigApply::new(
        values
            .iter()
            .map(|(key, fingerprint)| staged_value(key, *fingerprint))
            .collect(),
        ignore_nonexistent,
        hash_refreshed,
    )
}

#[test]
fn verify_update_existing_region_config_apply_replaces_matching_values() {
    let current_config = config_key(71);
    let derived_config = config_key(72);
    let diff = staged_diff(&["wall_loops", "top_shell_layers"]);
    let event = invalidate_event(
        current_config,
        derived_config,
        &["wall_loops", "top_shell_layers"],
    );
    let current = [
        staged_value("wall_loops", 1),
        staged_value("top_shell_layers", 3),
    ];
    let derived = [
        staged_value("wall_loops", 10),
        staged_value("top_shell_layers", 30),
    ];

    let apply =
        staged_verify_update_existing_region_config_apply(Some(&event), &current, &derived, &diff);

    assert_eq!(
        apply,
        Some(staged_apply(
            &[("wall_loops", 10), ("top_shell_layers", 30)],
            false,
            true
        ))
    );
}

#[test]
fn verify_update_existing_region_config_apply_processes_duplicate_diff_keys_in_order() {
    let current_config = config_key(71);
    let derived_config = config_key(72);
    let diff = staged_diff(&["wall_loops", "top_shell_layers", "wall_loops"]);
    let event = invalidate_event(
        current_config,
        derived_config,
        &["wall_loops", "top_shell_layers", "wall_loops"],
    );
    let current = [
        staged_value("wall_loops", 1),
        staged_value("top_shell_layers", 3),
    ];
    let derived = [
        staged_value("wall_loops", 10),
        staged_value("top_shell_layers", 30),
    ];

    let apply =
        staged_verify_update_existing_region_config_apply(Some(&event), &current, &derived, &diff);

    assert_eq!(
        apply,
        Some(staged_apply(
            &[("wall_loops", 10), ("top_shell_layers", 30)],
            false,
            true
        ))
    );
}

#[test]
fn verify_update_existing_region_config_apply_ignores_diff_keys_missing_from_derived() {
    let current_config = config_key(71);
    let derived_config = config_key(72);
    let diff = staged_diff(&["wall_loops", "missing"]);
    let event = invalidate_event(current_config, derived_config, &["wall_loops", "missing"]);
    let current = [staged_value("wall_loops", 1), staged_value("missing", 5)];
    let derived = [staged_value("wall_loops", 10)];

    let apply =
        staged_verify_update_existing_region_config_apply(Some(&event), &current, &derived, &diff);

    assert_eq!(
        apply,
        Some(staged_apply(
            &[("wall_loops", 10), ("missing", 5)],
            false,
            true
        ))
    );
}

#[test]
fn verify_update_existing_region_config_apply_requires_invalidate_event() {
    let diff = staged_diff(&["wall_loops"]);
    let current = [staged_value("wall_loops", 1)];
    let derived = [staged_value("wall_loops", 10)];

    let apply = staged_verify_update_existing_region_config_apply(None, &current, &derived, &diff);

    assert_eq!(apply, None);
}

#[test]
fn verify_update_existing_region_config_apply_records_ignore_nonexistent_false() {
    let current_config = config_key(71);
    let derived_config = config_key(72);
    let diff = staged_diff(&["wall_loops"]);
    let event = invalidate_event(current_config, derived_config, &["wall_loops"]);
    let current = [staged_value("wall_loops", 1)];
    let derived = [staged_value("wall_loops", 10)];

    let apply =
        staged_verify_update_existing_region_config_apply(Some(&event), &current, &derived, &diff);

    assert_eq!(
        apply,
        Some(staged_apply(&[("wall_loops", 10)], false, true))
    );
}

#[test]
fn verify_update_existing_region_config_apply_records_hash_refreshed() {
    let current_config = config_key(71);
    let derived_config = config_key(72);
    let diff = staged_diff(&["wall_loops"]);
    let event = invalidate_event(current_config, derived_config, &["wall_loops"]);
    let current = [staged_value("wall_loops", 1)];
    let derived = [staged_value("wall_loops", 10)];

    let apply =
        staged_verify_update_existing_region_config_apply(Some(&event), &current, &derived, &diff);

    assert_eq!(
        apply,
        Some(staged_apply(&[("wall_loops", 10)], false, true))
    );
}
