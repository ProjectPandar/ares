use super::super::apply_normalization_state::{
    StagedApplyNormalizationCall, staged_apply_normalization_prelude,
};

fn changed_keys(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

#[test]
fn apply_normalization_materializes_profile_ids_in_source_order() {
    let normalized = staged_apply_normalization_prelude(0, &[], &[]);

    assert_eq!(
        normalized.materialized_profile_id_keys(),
        &[
            "print_settings_id",
            "filament_settings_id",
            "printer_settings_id"
        ]
    );
}

#[test]
fn apply_normalization_preserves_used_filament_order_and_builds_set() {
    let used_filaments = vec![1, 3, 1, 2];
    let normalized = staged_apply_normalization_prelude(0, &used_filaments, &[]);

    assert_eq!(normalized.used_filaments(), used_filaments.as_slice());
    assert_eq!(normalized.used_filament_set().len(), 3);
    assert!(normalized.used_filament_set().contains(&1));
    assert!(normalized.used_filament_set().contains(&2));
    assert!(normalized.used_filament_set().contains(&3));
}

#[test]
fn apply_normalization_calls_normalize_fdm_1_before_normalize_fdm_2() {
    let normalized = staged_apply_normalization_prelude(2, &[1], &[]);

    assert_eq!(
        normalized.calls(),
        &[
            StagedApplyNormalizationCall::NormalizeFdm1,
            StagedApplyNormalizationCall::NormalizeFdm2 {
                object_count: 2,
                used_filament_count: 1,
            },
        ]
    );
}

#[test]
fn apply_normalization_passes_object_count_and_used_filament_count() {
    let used_filaments = vec![4, 4, 8];
    let normalized = staged_apply_normalization_prelude(7, &used_filaments, &[]);

    assert_eq!(
        normalized.calls()[1],
        StagedApplyNormalizationCall::NormalizeFdm2 {
            object_count: 7,
            used_filament_count: 3,
        }
    );
}

#[test]
fn apply_normalization_preserves_changed_keys_in_order() {
    let changed_keys = changed_keys(&["enable_prime_tower", "independent_support_layer_height"]);
    let normalized = staged_apply_normalization_prelude(1, &[], &changed_keys);

    assert_eq!(normalized.changed_keys(), changed_keys.as_slice());
}

#[test]
fn apply_normalization_empty_filaments_passes_zero_and_empty_set() {
    let normalized = staged_apply_normalization_prelude(4, &[], &[]);

    assert!(normalized.used_filaments().is_empty());
    assert!(normalized.used_filament_set().is_empty());
    assert_eq!(
        normalized.calls()[1],
        StagedApplyNormalizationCall::NormalizeFdm2 {
            object_count: 4,
            used_filament_count: 0,
        }
    );
}
