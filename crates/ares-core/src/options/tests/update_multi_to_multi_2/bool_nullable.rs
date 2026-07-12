use super::{options, update};
use crate::SliceError;
use serde_json::json;

#[test]
fn bool_baseline_is_overwritten_by_first_non_nil_same_variant_source() {
    let mut target = options(json!({
        "filament_long_retractions_when_cut": ["nil", true, false]
    }));
    let dst_config = options(json!({
        "filament_long_retractions_when_cut": [false, true, true]
    }));

    update(
        &mut target,
        &dst_config,
        &["filament_long_retractions_when_cut"],
    )
    .unwrap();

    assert_eq!(
        target.values()["filament_long_retractions_when_cut"],
        json!([true, false, true])
    );
}

#[test]
fn bool_duplicate_source_variants_skip_nil_and_select_first_non_nil_false() {
    let mut target = options(json!({
        "filament_long_retractions_when_cut": ["nil", false, true]
    }));
    let dst_config = options(json!({
        "filament_long_retractions_when_cut": [true, false, true]
    }));

    update(
        &mut target,
        &dst_config,
        &["filament_long_retractions_when_cut"],
    )
    .unwrap();

    assert_eq!(
        target.values()["filament_long_retractions_when_cut"],
        json!([false, true, true])
    );
}

#[test]
fn bool_missing_source_variant_does_not_fallback_to_all_source_indices() {
    let mut target = options(json!({
        "filament_long_retractions_when_cut": [true, false, true]
    }));
    let dst_config = options(json!({
        "filament_long_retractions_when_cut": [false, false, false]
    }));

    update(
        &mut target,
        &dst_config,
        &["filament_long_retractions_when_cut"],
    )
    .unwrap();

    assert_eq!(
        target.values()["filament_long_retractions_when_cut"],
        json!([true, true, false])
    );
}

#[test]
fn bool_all_nil_matches_preserve_destination_values_and_nil() {
    let mut target = options(json!({
        "filament_long_retractions_when_cut": ["nil", "nil", false]
    }));
    let dst_config = options(json!({
        "filament_long_retractions_when_cut": [true, "nil", "nil"]
    }));

    update(
        &mut target,
        &dst_config,
        &["filament_long_retractions_when_cut"],
    )
    .unwrap();

    assert_eq!(
        target.values()["filament_long_retractions_when_cut"],
        json!([true, false, "nil"])
    );
}

#[test]
fn bool_tokens_accept_upstream_numeric_forms_and_reject_true_false_strings() {
    let mut target = options(json!({
        "filament_long_retractions_when_cut": ["1", 0, "nil"]
    }));
    let dst_config = options(json!({
        "filament_long_retractions_when_cut": [false, true, false]
    }));

    update(
        &mut target,
        &dst_config,
        &["filament_long_retractions_when_cut"],
    )
    .unwrap();

    assert_eq!(
        target.values()["filament_long_retractions_when_cut"],
        json!([true, true, false])
    );

    let mut invalid = options(json!({
        "filament_long_retractions_when_cut": [true, "false", false]
    }));
    let before = invalid.clone();

    let result = update(
        &mut invalid,
        &dst_config,
        &["filament_long_retractions_when_cut"],
    );

    assert!(matches!(result, Err(SliceError::InvalidInput(_))));
    assert_eq!(invalid, before);
}

#[test]
fn invalid_bool_values_and_lengths_do_not_mutate() {
    let cases = [
        (
            json!({"filament_long_retractions_when_cut": [true, false, true]}),
            json!({}),
        ),
        (
            json!({"filament_long_retractions_when_cut": [true, 2, true]}),
            json!({"filament_long_retractions_when_cut": [false, false, false]}),
        ),
        (
            json!({"filament_long_retractions_when_cut": [true, false, true]}),
            json!({"filament_long_retractions_when_cut": [false, "yes", false]}),
        ),
        (
            json!({"filament_long_retractions_when_cut": [true, false]}),
            json!({"filament_long_retractions_when_cut": [false, false, false]}),
        ),
        (
            json!({"filament_long_retractions_when_cut": [true, false, true]}),
            json!({"filament_long_retractions_when_cut": [false, false]}),
        ),
    ];

    for (target_value, dst_value) in cases {
        let mut target = options(target_value);
        let before = target.clone();
        let dst_config = options(dst_value);

        let result = update(
            &mut target,
            &dst_config,
            &["filament_long_retractions_when_cut"],
        );

        assert!(matches!(result, Err(SliceError::InvalidInput(_))));
        assert_eq!(target, before);
    }
}

#[test]
fn later_invalid_bool_key_does_not_partially_mutate_earlier_valid_key() {
    let mut target = options(json!({
        "filament_long_retractions_when_cut": ["nil", true, false],
        "filament_retract_when_changing_layer": [true, false, true]
    }));
    let before = target.clone();
    let dst_config = options(json!({
        "filament_long_retractions_when_cut": [false, false, false],
        "filament_retract_when_changing_layer": [false, "bad", false]
    }));

    let result = update(
        &mut target,
        &dst_config,
        &[
            "filament_long_retractions_when_cut",
            "filament_retract_when_changing_layer",
        ],
    );

    assert!(matches!(result, Err(SliceError::InvalidInput(_))));
    assert_eq!(target, before);
}

#[test]
fn representative_bool_option_names_merge() {
    let mut target = options(json!({
        "filament_long_retractions_when_cut": ["nil", true, false],
        "filament_retract_when_changing_layer": [false, "nil", true],
        "filament_wipe": [true, false, "nil"]
    }));
    let dst_config = options(json!({
        "filament_long_retractions_when_cut": [false, false, false],
        "filament_retract_when_changing_layer": [true, true, true],
        "filament_wipe": [false, true, false]
    }));

    update(
        &mut target,
        &dst_config,
        &[
            "filament_long_retractions_when_cut",
            "filament_retract_when_changing_layer",
            "filament_wipe",
        ],
    )
    .unwrap();

    assert_eq!(
        target.values()["filament_long_retractions_when_cut"],
        json!([true, false, false])
    );
    assert_eq!(
        target.values()["filament_retract_when_changing_layer"],
        json!([false, true, true])
    );
    assert_eq!(target.values()["filament_wipe"], json!([true, true, false]));
}
