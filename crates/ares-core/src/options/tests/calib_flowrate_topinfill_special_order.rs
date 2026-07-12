use crate::{SliceError, SliceOptions};
use serde_json::json;

#[test]
fn calib_flowrate_topinfill_special_order_defaults_false() {
    let options: SliceOptions = serde_json::from_value(json!({})).unwrap();

    assert!(
        !options
            .infill_options()
            .unwrap()
            .calib_flowrate_topinfill_special_order()
    );
}

#[test]
fn parses_calib_flowrate_topinfill_special_order_bool_values() {
    for value in [false, true] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "calib_flowrate_topinfill_special_order": value }))
                .unwrap();

        assert_eq!(
            options
                .infill_options()
                .unwrap()
                .calib_flowrate_topinfill_special_order(),
            value
        );
    }
}

#[test]
fn rejects_non_bool_calib_flowrate_topinfill_special_order() {
    for value in [
        json!(0),
        json!(1),
        json!("true"),
        json!("false"),
        json!(null),
        json!([]),
        json!({"enabled": true}),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "calib_flowrate_topinfill_special_order": value
        }))
        .unwrap();

        let err = options.infill_options().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(
            err.to_string()
                .contains("calib_flowrate_topinfill_special_order"),
            "{err}"
        );
    }
}
