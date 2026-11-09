use ares_core::{Nullable, ProcessOptions};
use serde_json::{Value, json};

fn infill_speed(input: Value) -> Result<Vec<Option<f64>>, serde_json::Error> {
    let options: ProcessOptions =
        serde_json::from_value(json!({"initial_layer_infill_speed": input}))?;
    Ok(options
        .print
        .initial_layer_infill_speed
        .0
        .into_iter()
        .map(|value| match value {
            Nullable::Value(value) => Some(value.0),
            Nullable::Nil => None,
        })
        .collect())
}

// Config.hpp:932-958: getline, trim, nil, then unchecked double extraction.
#[test]
fn process_cofloats_numeric_prefixes() {
    for (wire, value) in [
        ("35%", 35.0),
        ("35units", 35.0),
        ("-3.5e1%", -35.0),
        (".35E2 mm/s", 35.0),
        (" 35 % ", 35.0),
        ("\u{b}35\u{b}", 35.0),
        ("\u{a0}35", 0.0),
        ("bogus", 0.0),
        ("nan", 0.0),
        ("inf", 0.0),
        ("1e+", 0.0),
        ("1e999", f64::MAX),
        ("-1e999", -f64::MAX),
        ("1e-999", 0.0),
        ("0x1p2", 0.0),
    ] {
        for input in [json!(wire), json!([wire])] {
            assert_eq!(
                infill_speed(input.clone()).unwrap(),
                vec![Some(value)],
                "{input}"
            );
        }
    }
}

#[test]
fn process_cofloats_wire_slots() {
    for input in [
        json!("35%,nil,4.7e1units,"),
        json!(["35%,nil", "4.7e1units,"]),
    ] {
        assert_eq!(
            infill_speed(input).unwrap(),
            vec![Some(35.0), None, Some(47.0)]
        );
    }
    for input in [json!(""), json!([])] {
        assert_eq!(infill_speed(input).unwrap(), vec![Some(0.0)]);
    }
    assert_eq!(
        infill_speed(json!([35.0, "nil", 47.0])).unwrap(),
        vec![Some(35.0), None, Some(47.0)]
    );
    assert!(infill_speed(json!([35.0, null, 47.0])).is_err());
}

#[test]
fn process_cofloats_empty_interior_has_no_numeric_value() {
    for input in [json!(" "), json!("35,,47"), json!(["35", " ", "47"])] {
        assert!(infill_speed(input).is_err());
    }
}
