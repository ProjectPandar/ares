use super::super::*;
use serde_json::json;

#[test]
fn normalizes_legacy_alias_keys_after_rotation_slice() {
    let options: SliceOptions = serde_json::from_value(json!({
        "sparse_infill_anchor": "12mm",
        "sparse_infill_anchor_max": "120%",
        "chamber_temperatures": [45],
        "thumbnail_size": "256x256",
        "initial_layer_flow_ratio": 1.05,
        "future_orca_key": "preserved"
    }))
    .unwrap();

    for legacy_key in [
        "sparse_infill_anchor",
        "sparse_infill_anchor_max",
        "chamber_temperatures",
        "thumbnail_size",
        "initial_layer_flow_ratio",
    ] {
        assert!(!options.values().contains_key(legacy_key));
    }
    assert_eq!(options.values()["infill_anchor"], json!("12mm"));
    assert_eq!(options.values()["infill_anchor_max"], json!("120%"));
    assert_eq!(options.values()["chamber_temperature"], json!([45]));
    assert_eq!(options.values()["thumbnails"], json!("256x256/PNG"));
    assert_eq!(
        options.values()["bottom_solid_infill_flow_ratio"],
        json!(1.05)
    );
    assert_eq!(options.values()["future_orca_key"], json!("preserved"));
}

#[test]
fn preserves_non_string_values_for_legacy_alias_keys() {
    let options: SliceOptions = serde_json::from_value(json!({
        "sparse_infill_anchor": true,
        "sparse_infill_anchor_max": 7,
        "thumbnail_size": [16, 16]
    }))
    .unwrap();

    assert_eq!(options.values()["infill_anchor"], json!(true));
    assert_eq!(options.values()["infill_anchor_max"], json!(7));
    assert_eq!(options.values()["thumbnails"], json!([16, 16]));
}

#[test]
fn normalizes_legacy_top_one_wall_type_only_when_not_none() {
    for legacy_value in ["all", "top", "1"] {
        let options: SliceOptions = serde_json::from_value(json!({
            "top_one_wall_type": legacy_value
        }))
        .unwrap();

        assert!(!options.values().contains_key("top_one_wall_type"));
        assert_eq!(options.values()["only_one_wall_top"], json!(true));
    }

    let none: SliceOptions = serde_json::from_value(json!({
        "top_one_wall_type": "none"
    }))
    .unwrap();
    assert_eq!(none.values()["top_one_wall_type"], json!("none"));
    assert!(!none.values().contains_key("only_one_wall_top"));

    for legacy_value in [
        json!(true),
        json!(7),
        json!(null),
        json!(["all"]),
        json!({"mode": "all"}),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "top_one_wall_type": legacy_value.clone()
        }))
        .unwrap();

        assert_eq!(options.values()["top_one_wall_type"], legacy_value);
        assert!(!options.values().contains_key("only_one_wall_top"));
    }
}
