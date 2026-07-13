use serde_json::{Value, json};

use super::super::super::{ProcessGCodeSourceOptions, ProcessOptions, ProcessPrintSourceOptions};

#[test]
fn remaining_arrays_round_trip_arbitrary_valid_lengths_in_children_and_parent() {
    for values in [json!([]), json!(["one", "two"]), json!(["a", "b", "c", "d", "e"])] {
        assert_round_trip::<ProcessPrintSourceOptions>("post_process", values.clone());
        assert_round_trip::<ProcessOptions>("post_process", values);
    }
    for values in [json!([]), json!(["0,0", "1,1"]), json!(["a", "b", "c", "d", "e"])] {
        assert_round_trip::<ProcessGCodeSourceOptions>(
            "small_area_infill_flow_compensation_model",
            values.clone(),
        );
        assert_round_trip::<ProcessOptions>("small_area_infill_flow_compensation_model", values);
    }
    for values in [json!([]), json!(["1", "2"]), json!(["1", "2", "3", "4", "5"])] {
        assert_round_trip::<ProcessPrintSourceOptions>("wiping_volumes_extruders", values.clone());
        assert_round_trip::<ProcessOptions>("wiping_volumes_extruders", values);
    }
}

#[test]
fn remaining_arrays_reject_scalar_null_and_invalid_element_shapes() {
    for (key, invalids) in [
        ("post_process", vec![json!("M117"), Value::Null, json!([7]), json!([{}])]),
        (
            "small_area_infill_flow_compensation_model",
            vec![json!("0,0"), Value::Null, json!([7]), json!([{}])],
        ),
        (
            "wiping_volumes_extruders",
            vec![json!("70"), Value::Null, json!(["bad"]), json!([{}])],
        ),
    ] {
        for invalid in invalids {
            let input = json!({key: invalid});
            let error = serde_json::from_value::<ProcessOptions>(input).unwrap_err().to_string();
            assert!(error.contains(key), "{key}: {error}");
        }
    }
}

fn assert_round_trip<T>(key: &str, values: Value)
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    let input = json!({key: values});
    let parsed: T = serde_json::from_value(input.clone()).unwrap();
    assert_eq!(serde_json::to_value(parsed).unwrap()[key], input[key]);
}
