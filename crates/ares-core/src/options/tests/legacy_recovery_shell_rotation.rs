use super::super::*;
use serde_json::json;

#[test]
fn normalizes_legacy_power_loss_recovery_values() {
    for value in ["1", "true", "TRUE", "True"] {
        let options: SliceOptions = serde_json::from_value(json!({
            "enable_power_loss_recovery": value,
            "future_orca_key": "preserved"
        }))
        .unwrap();
        assert_eq!(
            options.values()["enable_power_loss_recovery"],
            json!("enable")
        );
        assert_eq!(options.values()["future_orca_key"], json!("preserved"));
    }

    for value in ["0", "false", "FALSE", "False"] {
        let options: SliceOptions = serde_json::from_value(json!({
            "enable_power_loss_recovery": value
        }))
        .unwrap();
        assert_eq!(
            options.values()["enable_power_loss_recovery"],
            json!("disable")
        );
    }

    let unchanged: SliceOptions = serde_json::from_value(json!({
        "enable_power_loss_recovery": "maybe",
        "power_loss_non_string": true
    }))
    .unwrap();
    assert_eq!(
        unchanged.values()["enable_power_loss_recovery"],
        json!("maybe")
    );
    assert_eq!(unchanged.values()["power_loss_non_string"], json!(true));

    let non_string: SliceOptions = serde_json::from_value(json!({
        "enable_power_loss_recovery": true
    }))
    .unwrap();
    assert_eq!(
        non_string.values()["enable_power_loss_recovery"],
        json!(true)
    );
}

#[test]
fn normalizes_legacy_vertical_shell_thickness_values() {
    let all: SliceOptions = serde_json::from_value(json!({
        "ensure_vertical_shell_thickness": "1"
    }))
    .unwrap();
    assert_eq!(
        all.values()["ensure_vertical_shell_thickness"],
        json!("ensure_all")
    );

    let moderate: SliceOptions = serde_json::from_value(json!({
        "ensure_vertical_shell_thickness": "0"
    }))
    .unwrap();
    assert_eq!(
        moderate.values()["ensure_vertical_shell_thickness"],
        json!("ensure_moderate")
    );

    let unchanged: SliceOptions = serde_json::from_value(json!({
        "ensure_vertical_shell_thickness": "ensure_all"
    }))
    .unwrap();
    assert_eq!(
        unchanged.values()["ensure_vertical_shell_thickness"],
        json!("ensure_all")
    );

    let non_string: SliceOptions = serde_json::from_value(json!({
        "ensure_vertical_shell_thickness": true
    }))
    .unwrap();
    assert_eq!(
        non_string.values()["ensure_vertical_shell_thickness"],
        json!(true)
    );
}

#[test]
fn normalizes_legacy_rotate_solid_infill_direction_values() {
    for (legacy_value, expected) in [
        (json!("1"), json!("0,90")),
        (json!("0"), json!("0")),
        (json!("45"), json!("45")),
        (json!(true), json!(true)),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "rotate_solid_infill_direction": legacy_value
        }))
        .unwrap();

        assert!(
            !options
                .values()
                .contains_key("rotate_solid_infill_direction")
        );
        assert_eq!(options.values()["solid_infill_rotate_template"], expected);
    }
}
