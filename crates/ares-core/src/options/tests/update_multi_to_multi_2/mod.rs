use crate::{MultiToMulti2Update, SliceError, SliceOptions};
use serde_json::{Value, json};

fn options(value: Value) -> SliceOptions {
    serde_json::from_value(value).unwrap()
}

fn update(
    target: &mut SliceOptions,
    dst_config: &SliceOptions,
    keys: &[&str],
) -> Result<isize, SliceError> {
    target.update_values_from_multi_to_multi_2_float_keys(MultiToMulti2Update {
        src_extruder_variants: &[
            "Direct Drive Standard",
            "Direct Drive Standard",
            "Bowden Standard",
        ],
        dst_extruder_variants: &[
            "Direct Drive Standard",
            "Bowden Standard",
            "Missing Variant",
        ],
        dst_config,
        key_set: keys,
    })
}

#[test]
fn only_present_source_keys_in_key_set_are_processed() {
    let mut target = options(json!({
        "filament_flow_ratio": [0.9, 0.8, 0.7]
    }));
    let dst_config = options(json!({
        "filament_flow_ratio": [1.0, 1.1, 1.2],
        "filament_retraction_length": [4.0, 5.0, 6.0]
    }));

    update(
        &mut target,
        &dst_config,
        &["filament_flow_ratio", "filament_retraction_length"],
    )
    .unwrap();

    assert_eq!(
        target.values()["filament_flow_ratio"],
        json!([0.8, 0.7, 1.2])
    );
    assert!(!target.values().contains_key("filament_retraction_length"));
}

#[test]
fn destination_baseline_is_overwritten_by_minimum_non_nil_same_variant_source() {
    let mut target = options(json!({
        "filament_flow_ratio": [0.9, 0.6, 0.7]
    }));
    let dst_config = options(json!({
        "filament_flow_ratio": [1.0, 1.1, 1.2]
    }));

    update(&mut target, &dst_config, &["filament_flow_ratio"]).unwrap();

    assert_eq!(
        target.values()["filament_flow_ratio"],
        json!([0.6, 0.7, 1.2])
    );
}

#[test]
fn duplicate_source_variants_ignore_nil_entries() {
    let mut target = options(json!({
        "filament_flow_ratio": ["nil", 0.6, "nil"]
    }));
    let dst_config = options(json!({
        "filament_flow_ratio": [1.0, 1.1, 1.2]
    }));

    update(&mut target, &dst_config, &["filament_flow_ratio"]).unwrap();

    assert_eq!(
        target.values()["filament_flow_ratio"],
        json!([0.6, 1.1, 1.2])
    );
}

#[test]
fn missing_source_variant_does_not_fallback_to_all_source_indices() {
    let mut target = options(json!({
        "filament_flow_ratio": [0.3, 0.4, 0.5]
    }));
    let dst_config = options(json!({
        "filament_flow_ratio": [1.0, 1.1, 1.2]
    }));

    update(&mut target, &dst_config, &["filament_flow_ratio"]).unwrap();

    assert_eq!(
        target.values()["filament_flow_ratio"],
        json!([0.3, 0.5, 1.2])
    );
}

#[test]
fn all_nil_matching_source_entries_preserve_destination_values() {
    let mut target = options(json!({
        "filament_flow_ratio": ["nil", "nil", 0.5]
    }));
    let dst_config = options(json!({
        "filament_flow_ratio": [1.0, 1.1, 1.2]
    }));

    update(&mut target, &dst_config, &["filament_flow_ratio"]).unwrap();

    assert_eq!(
        target.values()["filament_flow_ratio"],
        json!([1.0, 0.5, 1.2])
    );
}

#[test]
fn destination_nil_is_preserved_when_no_non_nil_source_match_exists() {
    let mut target = options(json!({
        "filament_flow_ratio": ["nil", "nil", 0.5]
    }));
    let dst_config = options(json!({
        "filament_flow_ratio": ["nil", "nil", "nil"]
    }));

    update(&mut target, &dst_config, &["filament_flow_ratio"]).unwrap();

    assert_eq!(
        target.values()["filament_flow_ratio"],
        json!(["nil", 0.5, "nil"])
    );
}

#[test]
fn missing_destination_invalid_values_and_length_mismatches_do_not_mutate() {
    let cases = [
        (json!({"filament_flow_ratio": [0.9, 0.8, 0.7]}), json!({})),
        (
            json!({"filament_flow_ratio": [0.9, "fast", 0.7]}),
            json!({"filament_flow_ratio": [1.0, 1.1, 1.2]}),
        ),
        (
            json!({"filament_flow_ratio": [0.9, 0.8, 0.7]}),
            json!({"filament_flow_ratio": [1.0, "fast", 1.2]}),
        ),
        (
            json!({"filament_flow_ratio": [0.9, 0.8]}),
            json!({"filament_flow_ratio": [1.0, 1.1, 1.2]}),
        ),
        (
            json!({"filament_flow_ratio": [0.9, 0.8, 0.7]}),
            json!({"filament_flow_ratio": [1.0, 1.1]}),
        ),
    ];

    for (target_value, dst_value) in cases {
        let mut target = options(target_value);
        let before = target.clone();
        let dst_config = options(dst_value);

        let result = update(&mut target, &dst_config, &["filament_flow_ratio"]);

        assert!(matches!(result, Err(SliceError::InvalidInput(_))));
        assert_eq!(target, before);
    }
}

#[test]
fn unsupported_kinds_are_skipped() {
    let mut target = options(json!({
        "filament_colour": ["#111111", "#222222", "#333333"]
    }));
    let dst_config = options(json!({
        "filament_colour": ["#aaaaaa", "#bbbbbb", "#cccccc"]
    }));

    update(&mut target, &dst_config, &["filament_colour"]).unwrap();

    assert_eq!(
        target.values()["filament_colour"],
        json!(["#111111", "#222222", "#333333"])
    );
}

#[test]
fn representative_nullable_and_non_nullable_float_option_names_merge() {
    let mut target = options(json!({
        "filament_flow_ratio": [0.9, 0.8, 0.7],
        "filament_retraction_length": ["nil", 2.0, 1.0],
        "fan_max_speed": [90.0, 70.0, 60.0]
    }));
    let dst_config = options(json!({
        "filament_flow_ratio": [1.0, 1.1, 1.2],
        "filament_retraction_length": [3.0, 4.0, 5.0],
        "fan_max_speed": [100.0, 100.0, 100.0]
    }));

    update(
        &mut target,
        &dst_config,
        &[
            "filament_flow_ratio",
            "filament_retraction_length",
            "fan_max_speed",
        ],
    )
    .unwrap();

    assert_eq!(
        target.values()["filament_flow_ratio"],
        json!([0.8, 0.7, 1.2])
    );
    assert_eq!(
        target.values()["filament_retraction_length"],
        json!([2.0, 1.0, 5.0])
    );
    assert_eq!(target.values()["fan_max_speed"], json!([70.0, 60.0, 100.0]));
}

#[test]
fn float_or_percent_baseline_is_overwritten_by_upstream_sentinel_candidate() {
    let mut target = options(json!({
        "bridge_acceleration": ["90%", 80.0, 70.0]
    }));
    let dst_config = options(json!({
        "bridge_acceleration": [150.0, "160%", 170.0]
    }));

    update(&mut target, &dst_config, &["bridge_acceleration"]).unwrap();

    assert_eq!(
        target.values()["bridge_acceleration"],
        json!([80.0, 70.0, 170.0])
    );
}

#[test]
fn float_or_percent_duplicate_source_variants_skip_nil_and_preserve_selected_percent_flag() {
    let mut target = options(json!({
        "bridge_acceleration": ["nil", "60%", "nil"]
    }));
    let dst_config = options(json!({
        "bridge_acceleration": [150.0, 160.0, 170.0]
    }));

    update(&mut target, &dst_config, &["bridge_acceleration"]).unwrap();

    assert_eq!(
        target.values()["bridge_acceleration"],
        json!(["60%", 160.0, 170.0])
    );
}

#[test]
fn float_or_percent_equal_numeric_source_values_keep_first_selected_candidate() {
    let mut target = options(json!({
        "bridge_acceleration": [50.0, "50%", "nil"]
    }));
    let dst_config = options(json!({
        "bridge_acceleration": [150.0, 160.0, 170.0]
    }));

    update(&mut target, &dst_config, &["bridge_acceleration"]).unwrap();

    assert_eq!(
        target.values()["bridge_acceleration"],
        json!([50.0, 160.0, 170.0])
    );
}

#[test]
fn float_or_percent_values_at_or_above_sentinel_write_upstream_sentinel() {
    let mut target = options(json!({
        "bridge_acceleration": [9999.0, "10000%", "nil"]
    }));
    let dst_config = options(json!({
        "bridge_acceleration": [150.0, 160.0, 170.0]
    }));

    update(&mut target, &dst_config, &["bridge_acceleration"]).unwrap();

    assert_eq!(
        target.values()["bridge_acceleration"],
        json!(["9999%", 160.0, 170.0])
    );
}

#[test]
fn float_or_percent_missing_source_variant_does_not_fallback_to_all_source_indices() {
    let mut target = options(json!({
        "bridge_acceleration": [10.0, 20.0, 30.0]
    }));
    let dst_config = options(json!({
        "bridge_acceleration": [150.0, 160.0, 170.0]
    }));

    update(&mut target, &dst_config, &["bridge_acceleration"]).unwrap();

    assert_eq!(
        target.values()["bridge_acceleration"],
        json!([10.0, 30.0, 170.0])
    );
}

#[test]
fn float_or_percent_all_nil_matches_preserve_destination_values_and_nil() {
    let mut target = options(json!({
        "bridge_acceleration": ["nil", "nil", 30.0]
    }));
    let dst_config = options(json!({
        "bridge_acceleration": [150.0, "nil", "nil"]
    }));

    update(&mut target, &dst_config, &["bridge_acceleration"]).unwrap();

    assert_eq!(
        target.values()["bridge_acceleration"],
        json!([150.0, 30.0, "nil"])
    );
}

#[test]
fn invalid_float_or_percent_values_and_lengths_do_not_mutate() {
    let cases = [
        (
            json!({"bridge_acceleration": [50.0, 60.0, 70.0]}),
            json!({}),
        ),
        (
            json!({"bridge_acceleration": [50.0, "fast", 70.0]}),
            json!({"bridge_acceleration": [150.0, 160.0, 170.0]}),
        ),
        (
            json!({"bridge_acceleration": [50.0, 60.0, 70.0]}),
            json!({"bridge_acceleration": [150.0, "fast", 170.0]}),
        ),
        (
            json!({"bridge_acceleration": [50.0, 60.0]}),
            json!({"bridge_acceleration": [150.0, 160.0, 170.0]}),
        ),
        (
            json!({"bridge_acceleration": [50.0, 60.0, 70.0]}),
            json!({"bridge_acceleration": [150.0, 160.0]}),
        ),
    ];

    for (target_value, dst_value) in cases {
        let mut target = options(target_value);
        let before = target.clone();
        let dst_config = options(dst_value);

        let result = update(&mut target, &dst_config, &["bridge_acceleration"]);

        assert!(matches!(result, Err(SliceError::InvalidInput(_))));
        assert_eq!(target, before);
    }
}

#[test]
fn representative_float_or_percent_option_names_merge() {
    let mut target = options(json!({
        "bridge_acceleration": ["90%", 80.0, 70.0],
        "line_width": [0.45, "40%", "nil"],
        "outer_wall_line_width": [0.3, 0.25, 0.2]
    }));
    let dst_config = options(json!({
        "bridge_acceleration": [150.0, 160.0, 170.0],
        "line_width": [0.5, 0.6, 0.7],
        "outer_wall_line_width": [0.4, 0.4, 0.4]
    }));

    update(
        &mut target,
        &dst_config,
        &["bridge_acceleration", "line_width", "outer_wall_line_width"],
    )
    .unwrap();

    assert_eq!(
        target.values()["bridge_acceleration"],
        json!([80.0, 70.0, 170.0])
    );
    assert_eq!(target.values()["line_width"], json!([0.45, 0.6, 0.7]));
    assert_eq!(
        target.values()["outer_wall_line_width"],
        json!([0.25, 0.2, 0.4])
    );
}

mod bool_nullable;
