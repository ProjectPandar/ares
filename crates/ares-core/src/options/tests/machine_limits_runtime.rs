use super::super::{MachineLimits, SliceOptions};
use serde_json::json;

#[test]
fn machine_limits_defaults_match_orca_registry_values() {
    let limits = SliceOptions::default().machine_limits().unwrap();

    assert_eq!(
        limits,
        MachineLimits {
            emit_to_gcode: true,
            max_acceleration: [1000.0, 1000.0, 500.0, 5000.0],
            max_speed: [500.0, 500.0, 12.0, 120.0],
            max_acceleration_extruding: 1500.0,
            max_acceleration_retracting: 1500.0,
            max_acceleration_travel: 0.0,
            max_jerk: [10.0, 10.0, 0.2, 2.5],
            max_junction_deviation: 0.01,
        }
    );
}

#[test]
fn machine_limits_parse_first_values_from_vectors() {
    let options: SliceOptions = serde_json::from_value(json!({
        "emit_machine_limits_to_gcode": false,
        "machine_max_acceleration_x": [111.4, 222.0],
        "machine_max_acceleration_y": "222.5;333.0",
        "machine_max_acceleration_z": "333.6",
        "machine_max_acceleration_e": 444.4,
        "machine_max_speed_x": "55.4,99.0",
        "machine_max_speed_y": [66.5, 11.0],
        "machine_max_speed_z": "7.6",
        "machine_max_speed_e": 88.4,
        "machine_max_acceleration_extruding": [901.2, 1.0],
        "machine_max_acceleration_retracting": "802.5;1",
        "machine_max_acceleration_travel": 703.6,
        "machine_max_jerk_x": [9.1, 1.0],
        "machine_max_jerk_y": "8.2;1",
        "machine_max_jerk_z": "0.33",
        "machine_max_jerk_e": 4.4,
        "machine_max_junction_deviation": [0.025, 0.3]
    }))
    .unwrap();

    let limits = options.machine_limits().unwrap();

    assert!(!limits.emit_to_gcode);
    assert_eq!(limits.max_acceleration, [111.4, 222.5, 333.6, 444.4]);
    assert_eq!(limits.max_speed, [55.4, 66.5, 7.6, 88.4]);
    assert_eq!(limits.max_acceleration_extruding, 901.2);
    assert_eq!(limits.max_acceleration_retracting, 802.5);
    assert_eq!(limits.max_acceleration_travel, 703.6);
    assert_eq!(limits.max_jerk, [9.1, 8.2, 0.33, 4.4]);
    assert_eq!(limits.max_junction_deviation, 0.025);
}

#[test]
fn machine_limits_reject_invalid_emit_flag() {
    let options: SliceOptions = serde_json::from_value(json!({
        "emit_machine_limits_to_gcode": "true"
    }))
    .unwrap();

    assert!(
        matches!(options.machine_limits(), Err(err) if err.to_string().contains("emit_machine_limits_to_gcode must be a boolean"))
    );
}

#[test]
fn machine_limits_reject_invalid_numeric_vectors() {
    for (key, value) in [
        ("machine_max_acceleration_x", json!([])),
        ("machine_max_acceleration_y", json!(-0.1)),
        ("machine_max_acceleration_z", json!("NaN")),
        ("machine_max_acceleration_e", json!("inf")),
        ("machine_max_speed_x", json!(true)),
        ("machine_max_speed_y", json!(null)),
        ("machine_max_speed_z", json!({})),
        ("machine_max_speed_e", json!("fast")),
        ("machine_max_acceleration_extruding", json!(-1)),
        ("machine_max_acceleration_retracting", json!("bad")),
        ("machine_max_acceleration_travel", json!(-1)),
        ("machine_max_jerk_x", json!(-0.1)),
        ("machine_max_jerk_y", json!("bad")),
        ("machine_max_jerk_z", json!([])),
        ("machine_max_jerk_e", json!({})),
        ("machine_max_junction_deviation", json!(-0.001)),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({ key: value })).unwrap();
        assert!(
            options.machine_limits().is_err(),
            "{key} should reject invalid machine limit value"
        );
    }
}
