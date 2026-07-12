use super::super::*;
use serde_json::json;

#[test]
fn normalizes_legacy_prime_tower_rib_wall_enabled_value() {
    let options: SliceOptions = serde_json::from_value(json!({
        "prime_tower_rib_wall": "1",
        "future_orca_key": "preserved"
    }))
    .unwrap();

    assert!(!options.values().contains_key("prime_tower_rib_wall"));
    assert_eq!(options.values()["wipe_tower_wall_type"], json!("rib"));
    assert_eq!(options.values()["future_orca_key"], json!("preserved"));
}

#[test]
fn drops_legacy_prime_tower_rib_wall_when_not_enabled() {
    for value in [
        json!("0"),
        json!("false"),
        json!(true),
        json!(1),
        json!(null),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "prime_tower_rib_wall": value
        }))
        .unwrap();

        assert!(!options.values().contains_key("prime_tower_rib_wall"));
        assert!(!options.values().contains_key("wipe_tower_wall_type"));
    }
}

#[test]
fn normalizes_legacy_prime_tower_and_hardware_aliases() {
    let options: SliceOptions = serde_json::from_value(json!({
        "prime_tower_extra_rib_length": 1.2,
        "prime_tower_rib_width": "0.8",
        "prime_tower_fillet_wall": true,
        "extruder_clearance_max_radius": 42,
        "machine_switch_extruder_time": "3.5"
    }))
    .unwrap();

    for legacy_key in [
        "prime_tower_extra_rib_length",
        "prime_tower_rib_width",
        "prime_tower_fillet_wall",
        "extruder_clearance_max_radius",
        "machine_switch_extruder_time",
    ] {
        assert!(!options.values().contains_key(legacy_key));
    }
    assert_eq!(options.values()["wipe_tower_extra_rib_length"], json!(1.2));
    assert_eq!(options.values()["wipe_tower_rib_width"], json!("0.8"));
    assert_eq!(options.values()["wipe_tower_fillet_wall"], json!(true));
    assert_eq!(options.values()["extruder_clearance_radius"], json!(42));
    assert_eq!(options.values()["machine_tool_change_time"], json!("3.5"));
}

#[test]
fn normalizes_legacy_wall_direction_auto_value() {
    let auto: SliceOptions = serde_json::from_value(json!({
        "wall_direction": "auto"
    }))
    .unwrap();
    assert_eq!(auto.values()["wall_direction"], json!("ccw"));

    for (legacy_value, expected) in [
        (json!("ccw"), json!("ccw")),
        (json!("cw"), json!("cw")),
        (json!("Auto"), json!("Auto")),
        (json!(true), json!(true)),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "wall_direction": legacy_value
        }))
        .unwrap();
        assert_eq!(options.values()["wall_direction"], expected);
    }
}
