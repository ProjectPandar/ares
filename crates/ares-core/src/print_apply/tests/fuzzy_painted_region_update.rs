use super::super::fuzzy_painted_region_state::{
    StagedFuzzyPaintedRegionConfigChange, StagedFuzzyPaintedRegionConfigDerivation,
    StagedFuzzyPaintedRegionParent, StagedFuzzySkinConfig, StagedFuzzySkinType,
    staged_fuzzy_painted_region_config_apply, staged_fuzzy_painted_region_config_change,
    staged_fuzzy_painted_region_config_diff, staged_fuzzy_painted_region_invalidate_event,
    staged_fuzzy_painted_region_update_gate,
};
use super::super::print_region_state::{
    StagedPrintRegionConfigKey, StagedPrintRegionRefCount, staged_print_region_ref_inc,
};
use super::super::verify_update_config_state::{
    StagedConfigValue, StagedExistingRegionConfigApply, StagedExistingRegionConfigDiff,
    StagedExistingRegionInvalidateEvent, StagedExistingRegionUpdateAction,
};

fn config(region_id: usize, marker: u64, fuzzy_skin: StagedFuzzySkinType) -> StagedFuzzySkinConfig {
    StagedFuzzySkinConfig::new(region_id, marker, fuzzy_skin)
}

fn derivation(
    fuzzy_region_id: usize,
    parent: StagedFuzzyPaintedRegionParent,
    destination_region_id: usize,
    config: StagedFuzzySkinConfig,
) -> StagedFuzzyPaintedRegionConfigDerivation {
    StagedFuzzyPaintedRegionConfigDerivation::new(
        fuzzy_region_id,
        parent,
        destination_region_id,
        config,
    )
}

fn fuzzy_change(
    derivation: StagedFuzzyPaintedRegionConfigDerivation,
    current_config: StagedFuzzySkinConfig,
    config_changed: bool,
) -> StagedFuzzyPaintedRegionConfigChange {
    StagedFuzzyPaintedRegionConfigChange::new(derivation, current_config, config_changed)
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
fn fuzzy_painted_region_update_gate_keeps_unchanged_zero_ref_region() {
    let current = config(10, 99, StagedFuzzySkinType::All);
    let derived = derivation(
        5,
        StagedFuzzyPaintedRegionParent::VolumeRegion(0),
        50,
        current,
    );
    let config_change = staged_fuzzy_painted_region_config_change(derived, current);
    let region = StagedPrintRegionRefCount::default();

    let action = staged_fuzzy_painted_region_update_gate(config_change, &region);

    assert_eq!(action, StagedExistingRegionUpdateAction::Unchanged);
}

#[test]
fn fuzzy_painted_region_update_gate_keeps_unchanged_referenced_region() {
    let current = config(10, 99, StagedFuzzySkinType::All);
    let derived = derivation(
        5,
        StagedFuzzyPaintedRegionParent::VolumeRegion(0),
        50,
        current,
    );
    let config_change = staged_fuzzy_painted_region_config_change(derived, current);
    let mut region = StagedPrintRegionRefCount::default();
    staged_print_region_ref_inc(&mut region);

    let action = staged_fuzzy_painted_region_update_gate(config_change, &region);

    assert_eq!(action, StagedExistingRegionUpdateAction::Unchanged);
}

#[test]
fn fuzzy_painted_region_update_gate_updates_changed_zero_ref_region() {
    let current = config(10, 99, StagedFuzzySkinType::External);
    let expected = config(10, 99, StagedFuzzySkinType::All);
    let derived = derivation(
        5,
        StagedFuzzyPaintedRegionParent::VolumeRegion(0),
        50,
        expected,
    );
    let config_change = staged_fuzzy_painted_region_config_change(derived, current);
    let region = StagedPrintRegionRefCount::default();

    let action = staged_fuzzy_painted_region_update_gate(config_change, &region);

    assert_eq!(action, StagedExistingRegionUpdateAction::UpdateInPlace);
}

#[test]
fn fuzzy_painted_region_update_gate_reslices_changed_referenced_region() {
    let current = config(10, 99, StagedFuzzySkinType::External);
    let expected = config(10, 99, StagedFuzzySkinType::All);
    let derived = derivation(
        5,
        StagedFuzzyPaintedRegionParent::VolumeRegion(0),
        50,
        expected,
    );
    let config_change = staged_fuzzy_painted_region_config_change(derived, current);
    let mut region = StagedPrintRegionRefCount::default();
    staged_print_region_ref_inc(&mut region);

    let action = staged_fuzzy_painted_region_update_gate(config_change, &region);

    assert_eq!(action, StagedExistingRegionUpdateAction::RequiresReslice);
}

#[test]
fn fuzzy_painted_region_config_change_preserves_region_parent_destination_and_configs() {
    let current = config(10, 99, StagedFuzzySkinType::External);
    let expected = config(10, 99, StagedFuzzySkinType::All);
    let parent = StagedFuzzyPaintedRegionParent::PaintedRegion(2);
    let derived = derivation(7, parent, 70, expected);

    let config_change = staged_fuzzy_painted_region_config_change(derived, current);

    assert_eq!(config_change, fuzzy_change(derived, current, true));
}

#[test]
fn fuzzy_painted_region_config_diff_preserves_current_key_order_for_update_in_place() {
    let current = [
        value("fuzzy_skin", 1),
        value("marker", 2),
        value("unchanged", 3),
    ];
    let derived = [
        value("marker", 20),
        value("fuzzy_skin", 10),
        value("unchanged", 3),
    ];

    let diff = staged_fuzzy_painted_region_config_diff(
        StagedExistingRegionUpdateAction::UpdateInPlace,
        &current,
        &derived,
    );

    assert_eq!(diff, staged_diff(&["fuzzy_skin", "marker"]));
}

#[test]
fn fuzzy_painted_region_config_apply_skips_unchanged_action() {
    let current_key = config_key(41);
    let derived_key = config_key(42);
    let current = [value("fuzzy_skin", 1)];
    let derived = [value("fuzzy_skin", 10)];
    let diff = staged_fuzzy_painted_region_config_diff(
        StagedExistingRegionUpdateAction::Unchanged,
        &current,
        &derived,
    );
    let event = staged_fuzzy_painted_region_invalidate_event(
        StagedExistingRegionUpdateAction::Unchanged,
        current_key,
        derived_key,
        &diff,
    );

    let apply = staged_fuzzy_painted_region_config_apply(event.as_ref(), &current, &derived, &diff);

    assert_eq!(diff, staged_diff(&[]));
    assert_eq!(event, None);
    assert_eq!(apply, None);
}

#[test]
fn fuzzy_painted_region_config_apply_skips_requires_reslice_action() {
    let current_key = config_key(51);
    let derived_key = config_key(52);
    let current = [value("fuzzy_skin", 1)];
    let derived = [value("fuzzy_skin", 10)];
    let diff = staged_fuzzy_painted_region_config_diff(
        StagedExistingRegionUpdateAction::RequiresReslice,
        &current,
        &derived,
    );
    let event = staged_fuzzy_painted_region_invalidate_event(
        StagedExistingRegionUpdateAction::RequiresReslice,
        current_key,
        derived_key,
        &diff,
    );

    let apply = staged_fuzzy_painted_region_config_apply(event.as_ref(), &current, &derived, &diff);

    assert_eq!(diff, staged_diff(&[]));
    assert_eq!(event, None);
    assert_eq!(apply, None);
}

#[test]
fn fuzzy_painted_region_invalidate_event_preserves_callback_payload_before_apply() {
    let current_key = config_key(61);
    let derived_key = config_key(62);
    let current = [value("fuzzy_skin", 1), value("marker", 2)];
    let derived = [value("fuzzy_skin", 10), value("marker", 20)];
    let diff = staged_fuzzy_painted_region_config_diff(
        StagedExistingRegionUpdateAction::UpdateInPlace,
        &current,
        &derived,
    );

    let event = staged_fuzzy_painted_region_invalidate_event(
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
            &["fuzzy_skin", "marker"]
        ))
    );
}

#[test]
fn fuzzy_painted_region_config_apply_requires_invalidate_event_and_records_apply_only_state() {
    let current_key = config_key(71);
    let derived_key = config_key(72);
    let current = [value("fuzzy_skin", 1), value("marker", 2)];
    let derived = [value("fuzzy_skin", 10), value("marker", 20)];
    let diff = staged_fuzzy_painted_region_config_diff(
        StagedExistingRegionUpdateAction::UpdateInPlace,
        &current,
        &derived,
    );
    let event = staged_fuzzy_painted_region_invalidate_event(
        StagedExistingRegionUpdateAction::UpdateInPlace,
        current_key,
        derived_key,
        &diff,
    );

    let without_event = staged_fuzzy_painted_region_config_apply(None, &current, &derived, &diff);
    let with_event =
        staged_fuzzy_painted_region_config_apply(event.as_ref(), &current, &derived, &diff);

    assert_eq!(without_event, None);
    assert_eq!(
        with_event,
        Some(staged_apply(&[("fuzzy_skin", 10), ("marker", 20)]))
    );
}
