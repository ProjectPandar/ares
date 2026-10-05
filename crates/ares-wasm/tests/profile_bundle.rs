//! WASM boundary for the preset bundle import and compatibility filtering
//! exports: errors carry the core message; valid inputs pass through.

#[test]
fn parse_preset_bundle_round_trips_json() {
    let preset = serde_json::json!({
        "type": "process",
        "name": "P",
        "print_settings_id": "GP1",
        "version": "1.0.0.0"
    });
    let parsed =
        ares_wasm::parse_preset_bundle_json(preset.to_string().as_bytes(), "p.json").unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&parsed).unwrap();
    assert_eq!(parsed["profiles"][0]["name"], "P");
}

#[test]
fn parse_preset_bundle_surfaces_core_error() {
    let error = ares_wasm::parse_preset_bundle_json(b"[", "p.json").unwrap_err();
    assert!(error.contains("invalid"), "{error}");
}

#[test]
fn filter_compatible_presets_rejects_unknown_printer() {
    let catalog = serde_json::json!({"vendors": [], "imports": []}).to_string();
    let error = ares_wasm::filter_compatible_presets_json(&catalog, "missing", None).unwrap_err();
    assert!(
        error.contains("printer preset 'missing' was not found"),
        "{error}"
    );
}
