use super::super::*;
use serde_json::json;

#[test]
fn infill_combination_defaults_match_orca() {
    let infill = SliceOptions::default().infill_options().unwrap();

    assert!(!infill.infill_combination());
    assert_eq!(infill.infill_combination_max_layer_height_mm(), 0.4);
}

#[test]
fn parses_infill_combination_and_percent_max_height() {
    let options: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": [0.6],
        "infill_combination": true,
        "infill_combination_max_layer_height": "50%"
    }))
    .unwrap();
    let infill = options.infill_options().unwrap();

    assert!(infill.infill_combination());
    assert_eq!(infill.infill_combination_max_layer_height_mm(), 0.3);
}

#[test]
fn parses_absolute_and_zero_infill_combination_max_height() {
    let absolute: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": [0.6],
        "infill_combination_max_layer_height": 0.32
    }))
    .unwrap();
    let zero: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": [0.6],
        "infill_combination_max_layer_height": 0
    }))
    .unwrap();

    assert_eq!(
        absolute
            .infill_options()
            .unwrap()
            .infill_combination_max_layer_height_mm(),
        0.32
    );
    assert_eq!(
        zero.infill_options()
            .unwrap()
            .infill_combination_max_layer_height_mm(),
        0.6
    );
}

#[test]
fn rejects_invalid_infill_combination_values() {
    for value in [json!("true"), json!(1), json!(null)] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "infill_combination": value })).unwrap();

        assert!(matches!(
            options.infill_options(),
            Err(SliceError::InvalidInput(message))
                if message.contains("infill_combination must be a boolean")
        ));
    }
}

#[test]
fn rejects_invalid_infill_combination_max_height_values() {
    for value in [json!(-0.1), json!("bad"), json!(null)] {
        let options: SliceOptions = serde_json::from_value(json!({
            "infill_combination_max_layer_height": value
        }))
        .unwrap();

        assert!(matches!(
            options.infill_options(),
            Err(SliceError::InvalidInput(message))
                if message.contains("infill_combination_max_layer_height")
        ));
    }
}
