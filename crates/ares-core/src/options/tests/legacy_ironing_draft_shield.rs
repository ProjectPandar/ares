use super::super::*;
use serde_json::json;

#[test]
fn normalizes_legacy_ironing_direction_alias() {
    let options: SliceOptions = serde_json::from_value(json!({
        "ironing_direction": "45",
        "future_orca_key": "preserved"
    }))
    .unwrap();

    assert!(!options.values().contains_key("ironing_direction"));
    assert_eq!(options.values()["ironing_angle"], json!("45"));
    assert_eq!(options.values()["future_orca_key"], json!("preserved"));

    let non_string: SliceOptions = serde_json::from_value(json!({
        "ironing_direction": [0, 90]
    }))
    .unwrap();
    assert!(!non_string.values().contains_key("ironing_direction"));
    assert_eq!(non_string.values()["ironing_angle"], json!([0, 90]));
}

#[test]
fn normalizes_negative_legacy_ironing_angle_values() {
    for legacy_value in ["-1", "-45", "-0.5"] {
        let options: SliceOptions = serde_json::from_value(json!({
            "ironing_angle": legacy_value
        }))
        .unwrap();
        assert_eq!(options.values()["ironing_angle"], json!("0"));
    }

    for (legacy_value, expected) in [
        (json!("0"), json!("0")),
        (json!("45"), json!("45")),
        (json!(true), json!(true)),
        (json!(-5), json!(-5)),
        (json!(null), json!(null)),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "ironing_angle": legacy_value
        }))
        .unwrap();
        assert_eq!(options.values()["ironing_angle"], expected);
    }
}

#[test]
fn normalizes_legacy_counterbore_spelling_alias() {
    let options: SliceOptions = serde_json::from_value(json!({
        "counterbole_hole_bridging": "enabled"
    }))
    .unwrap();

    assert!(!options.values().contains_key("counterbole_hole_bridging"));
    assert_eq!(
        options.values()["counterbore_hole_bridging"],
        json!("enabled")
    );

    let non_string: SliceOptions = serde_json::from_value(json!({
        "counterbole_hole_bridging": true
    }))
    .unwrap();
    assert_eq!(
        non_string.values()["counterbore_hole_bridging"],
        json!(true)
    );
}

#[test]
fn normalizes_legacy_limited_draft_shield_value() {
    let limited: SliceOptions = serde_json::from_value(json!({
        "draft_shield": "limited"
    }))
    .unwrap();
    assert_eq!(limited.values()["draft_shield"], json!("disabled"));

    for (legacy_value, expected) in [
        (json!("disabled"), json!("disabled")),
        (json!("enabled"), json!("enabled")),
        (json!(true), json!(true)),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "draft_shield": legacy_value
        }))
        .unwrap();
        assert_eq!(options.values()["draft_shield"], expected);
    }
}
