use super::super::*;
use serde_json::{Value, json};

#[test]
fn first_layer_bed_temperature_defaults_to_cool_plate_default() {
    assert_eq!(
        SliceOptions::default()
            .first_layer_bed_temperature()
            .unwrap(),
        FirstLayerBedTemperature::new(35)
    );
}

#[test]
fn first_layer_bed_temperature_uses_selected_bed_type_default() {
    for (bed_type, expected) in [
        ("Cool Plate", 35),
        ("Textured Cool Plate", 40),
        ("Engineering Plate", 45),
        ("High Temp Plate", 45),
        ("Textured PEI Plate", 45),
        ("Supertack Plate", 35),
        ("SuperTack Plate", 35),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "curr_bed_type": bed_type
        }))
        .unwrap();

        assert_eq!(
            options.first_layer_bed_temperature().unwrap(),
            FirstLayerBedTemperature::new(expected),
            "{bed_type}"
        );
    }
}

#[test]
fn first_layer_bed_temperature_accepts_integer_forms_for_selected_key() {
    for (value, expected) in [
        (json!(50), 50),
        (json!("51"), 51),
        (json!("52;53"), 53),
        (json!("54,55"), 55),
        (json!([56, 57]), 57),
        (json!(0), 0),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "curr_bed_type": "High Temp Plate",
            "hot_plate_temp_initial_layer": value
        }))
        .unwrap();

        assert_eq!(
            options.first_layer_bed_temperature().unwrap(),
            FirstLayerBedTemperature::new(expected),
            "{expected}"
        );
    }
}

#[test]
fn first_layer_bed_temperature_formula_can_select_first_filament_value() {
    let options: SliceOptions = serde_json::from_value(json!({
        "bed_temperature_formula": "by_first_filament",
        "curr_bed_type": "High Temp Plate",
        "hot_plate_temp_initial_layer": [56, 57]
    }))
    .unwrap();

    assert_eq!(
        options.first_layer_bed_temperature().unwrap(),
        FirstLayerBedTemperature::new(56)
    );
}

#[test]
fn first_layer_bed_temperature_rejects_invalid_selected_key_values() {
    for value in [
        json!(-1),
        json!(45.5),
        json!("45.5"),
        json!([45.5]),
        json!(""),
        json!("45;"),
        json!([]),
        json!(["45", "bad"]),
        json!([["45"]]),
        json!({"value": 45}),
        json!(true),
        Value::Null,
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "curr_bed_type": "High Temp Plate",
            "hot_plate_temp_initial_layer": value
        }))
        .unwrap();

        let err = options.first_layer_bed_temperature().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("hot_plate_temp_initial_layer"));
    }
}

#[test]
fn first_layer_bed_temperature_rejects_invalid_bed_type() {
    let options: SliceOptions = serde_json::from_value(json!({
        "curr_bed_type": "Unknown Plate"
    }))
    .unwrap();

    let err = options.first_layer_bed_temperature().unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("curr_bed_type"));
}
