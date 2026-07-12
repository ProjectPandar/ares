use super::super::*;
use serde_json::json;

#[test]
fn symmetric_infill_y_axis_defaults_false() {
    let infill = SliceOptions::default().infill_options().unwrap();

    assert!(!infill.symmetric_infill_y_axis());
}

#[test]
fn parses_symmetric_infill_y_axis_true() {
    let options: SliceOptions = serde_json::from_value(json!({
        "symmetric_infill_y_axis": true
    }))
    .unwrap();

    assert!(options.infill_options().unwrap().symmetric_infill_y_axis());
}

#[test]
fn rejects_non_bool_symmetric_infill_y_axis_values() {
    for value in [json!(1), json!("true"), json!(null), json!([]), json!({})] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "symmetric_infill_y_axis": value })).unwrap();

        assert!(matches!(
            options.infill_options(),
            Err(SliceError::InvalidInput(message))
                if message.contains("symmetric_infill_y_axis")
        ));
    }
}
