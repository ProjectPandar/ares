use super::super::*;
use serde_json::json;

#[test]
fn accepts_orca_print_flow_ratio_bounds() {
    for value in [
        json!(0.01),
        json!(1.0),
        json!(2.0),
        json!("0.01"),
        json!("1.0"),
        json!("2.0"),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "print_flow_ratio": value })).unwrap();
        assert!(options.extrusion_options().is_ok());
    }
}

#[test]
fn rejects_invalid_print_flow_ratio_values() {
    for value in [
        json!(0.0),
        json!(-0.1),
        json!(2.1),
        json!("0.0"),
        json!("-0.1"),
        json!("2.1"),
        json!("not-a-number"),
        json!("NaN"),
        json!("inf"),
        json!("-inf"),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "print_flow_ratio": value })).unwrap();
        assert!(matches!(
            options.extrusion_options(),
            Err(SliceError::InvalidInput(_))
        ));
    }
}
