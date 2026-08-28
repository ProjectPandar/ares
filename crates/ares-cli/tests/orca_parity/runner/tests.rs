use super::{PresetKind, apply_override};

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

    let actual = apply_override(&base, &overrides, PresetKind::Filament);

    assert_eq!(actual["filament_type"], serde_json::json!(["PETG"]));
    assert!(!actual.contains_key("post_process"));
}

#[test]
fn inherited_smoke_process_keys_are_inserted_only_into_process() {
    let base = serde_json::Map::new();
    let overrides = serde_json::from_value(serde_json::json!({
        "wall_generator": "classic",
        "bed_exclude_area": ["0x0"]
    }))
    .unwrap();

    let process = apply_override(&base, &overrides, PresetKind::Process);
    let machine = apply_override(&base, &overrides, PresetKind::Machine);
    let filament = apply_override(&base, &overrides, PresetKind::Filament);

    assert_eq!(process["wall_generator"], "classic");
    assert!(!process.contains_key("bed_exclude_area"));
    assert_eq!(machine["bed_exclude_area"], serde_json::json!(["0x0"]));
    assert!(filament.is_empty());
}
