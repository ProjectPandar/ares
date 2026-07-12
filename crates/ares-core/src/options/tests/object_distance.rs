use super::super::*;
use serde_json::json;

fn options(value: serde_json::Value) -> SliceOptions {
    serde_json::from_value(value).unwrap()
}

#[test]
fn sla_printer_technology_returns_six_millimeters_without_fff_options() {
    let options = options(json!({"printer_technology": "SLA"}));

    assert_eq!(options.min_object_distance().unwrap(), 6.0);
}

#[test]
fn missing_fff_clearance_or_print_sequence_returns_zero() {
    for value in [
        json!({}),
        json!({"extruder_clearance_radius": 40.0}),
        json!({"print_sequence": "by object"}),
    ] {
        let options = options(value);

        assert_eq!(options.min_object_distance().unwrap(), 0.0);
    }
}

#[test]
fn by_object_print_sequence_uses_at_least_six_millimeters() {
    for (radius, expected) in [(4.0, 6.0), (6.0, 6.0), (40.0, 40.0)] {
        let options = options(json!({
            "extruder_clearance_radius": radius,
            "print_sequence": "by object"
        }));

        assert_eq!(options.min_object_distance().unwrap(), expected);
    }

    let options = options(json!({
        "extruder_clearance_radius": "40",
        "print_sequence": "by object"
    }));

    assert_eq!(options.min_object_distance().unwrap(), 40.0);
}

#[test]
fn non_by_object_print_sequence_returns_six_millimeters_when_required_options_exist() {
    for sequence in ["by layer", "by default"] {
        let options = options(json!({
            "extruder_clearance_radius": 40.0,
            "print_sequence": sequence
        }));

        assert_eq!(options.min_object_distance().unwrap(), 6.0);
    }
}

#[test]
fn invalid_boundary_values_return_invalid_input() {
    for value in [
        json!({"printer_technology": 1}),
        json!({"print_sequence": 1}),
        json!({"extruder_clearance_radius": -0.1, "print_sequence": "by object"}),
        json!({"extruder_clearance_radius": "inf", "print_sequence": "by object"}),
        json!({"extruder_clearance_radius": "abc", "print_sequence": "by object"}),
        json!({"extruder_clearance_radius": [], "print_sequence": "by object"}),
        json!({"extruder_clearance_radius": 40.0, "print_sequence": 1}),
        json!({"extruder_clearance_radius": 40.0, "print_sequence": "sequential"}),
    ] {
        let options = options(value);

        assert!(matches!(
            options.min_object_distance(),
            Err(SliceError::InvalidInput(_))
        ));
    }
}
