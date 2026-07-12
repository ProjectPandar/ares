use super::super::*;
use serde_json::json;

#[test]
fn accepts_orca_overhang_flow_ratio_bounds() {
    for value in [
        json!(0.0),
        json!(2.0),
        json!("0.0"),
        json!("1.25"),
        json!("2.0"),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "overhang_flow_ratio": value
        }))
        .unwrap();
        assert!(options.extrusion_options().is_ok());
    }
}

#[test]
fn rejects_invalid_overhang_flow_ratio_even_when_gate_is_false() {
    for value in [
        json!(-0.1),
        json!(2.1),
        json!("not-a-number"),
        json!([1.0]),
        json!(true),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "set_other_flow_ratios": false,
            "overhang_flow_ratio": value
        }))
        .unwrap();

        assert!(matches!(
            options.extrusion_options(),
            Err(SliceError::InvalidInput(_))
        ));
    }
}
