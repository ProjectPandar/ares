use super::apply_override;

#[test]
fn overrides_only_touch_options_owned_by_the_preset() {
    let base = serde_json::from_value(serde_json::json!({
        "name": "filament",
        "filament_type": ["PLA"]
    }))
    .unwrap();
    let overrides = serde_json::from_value(serde_json::json!({
        "post_process": [],
        "filament_type": ["PETG"]
    }))
    .unwrap();

    let actual = apply_override(&base, &overrides);

    assert_eq!(actual["filament_type"], serde_json::json!(["PETG"]));
    assert!(!actual.contains_key("post_process"));
}
