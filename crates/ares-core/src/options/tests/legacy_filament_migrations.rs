use super::super::*;
use serde_json::json;

#[test]
fn normalizes_legacy_filament_map_mode_auto_value() {
    let options: SliceOptions = serde_json::from_value(json!({
        "filament_map_mode": "Auto",
        "future_orca_key": "preserved"
    }))
    .unwrap();

    assert_eq!(
        options.values()["filament_map_mode"],
        json!("Auto For Flush")
    );
    assert_eq!(options.values()["future_orca_key"], json!("preserved"));

    for (legacy_value, expected) in [
        (json!("Auto For Flush"), json!("Auto For Flush")),
        (json!("auto"), json!("auto")),
        (json!(true), json!(true)),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "filament_map_mode": legacy_value
        }))
        .unwrap();
        assert_eq!(options.values()["filament_map_mode"], expected);
    }
}

#[test]
fn normalizes_legacy_filament_type_asa_aero_tokens() {
    for (legacy_value, expected) in [
        ("ASA-Aero", "\"ASA-AERO\""),
        ("\"ASA-Aero\"", "\"ASA-AERO\""),
        ("PLA;ASA-Aero;PETG", "\"PLA\";\"ASA-AERO\";\"PETG\""),
        ("\"PLA\";\"ASA-Aero\";PETG", "\"PLA\";\"ASA-AERO\";\"PETG\""),
        ("ASA-Aero;", "\"ASA-AERO\""),
        (";ASA-Aero", "\"\";\"ASA-AERO\""),
        ("PLA;;ASA-Aero", "\"PLA\";\"\";\"ASA-AERO\""),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "filament_type": legacy_value
        }))
        .unwrap();
        assert_eq!(options.values()["filament_type"], json!(expected));
    }
}

#[test]
fn preserves_legacy_filament_type_when_no_asa_aero_token_exists() {
    for legacy_value in [
        "PLA;PETG",
        "\"PLA\";PETG",
        "ASA-AERO",
        "asa-aero",
        "ASA-AeroX",
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "filament_type": legacy_value
        }))
        .unwrap();
        assert_eq!(options.values()["filament_type"], json!(legacy_value));
    }
}

#[test]
fn preserves_non_string_legacy_filament_type_values() {
    for legacy_value in [json!(true), json!(3), json!(null), json!(["ASA-Aero"])] {
        let options: SliceOptions = serde_json::from_value(json!({
            "filament_type": legacy_value.clone()
        }))
        .unwrap();
        assert_eq!(options.values()["filament_type"], legacy_value);
    }
}
