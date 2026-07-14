use std::collections::BTreeSet;

use serde::Deserialize;
use serde_json::{Map, Value};

use super::super::{
    InputShaperType, MachineEnvelopeOptions, OrcaBool, OrcaFloat, OrcaFloats, PrinterOptions,
    ProjectSettings,
};

#[derive(Debug, Deserialize)]
struct InventoryRow {
    key: String,
    raw_scope: String,
    static_owner: String,
    option_type: String,
    default_serialized: String,
    wire_shape: String,
}

#[derive(Clone, Copy)]
struct ExpectedField {
    key: &'static str,
    option_type: &'static str,
    default: &'static str,
    wire_shape: &'static str,
    fixture: &'static str,
}

const EXPECTED_FIELDS: [ExpectedField; 28] = [
    field("emit_machine_limits_to_gcode", "coBool", "1", "scalar_string", "\"1\""),
    field("input_shaping_damp_x", "coFloat", "0.1", "scalar_string", "\"0.1\""),
    field("input_shaping_damp_y", "coFloat", "0.1", "scalar_string", "\"0.1\""),
    field("input_shaping_emit", "coBool", "0", "scalar_string", "\"0\""),
    field("input_shaping_freq_x", "coFloat", "0", "scalar_string", "\"0\""),
    field("input_shaping_freq_y", "coFloat", "0", "scalar_string", "\"0\""),
    field("input_shaping_type", "coEnum", "Default", "scalar_string", "\"Default\""),
    field("machine_max_acceleration_e", "coFloats", "5000,5000", "array", "[\"30000\",\"5000\",\"30000\",\"5000\",\"5000\",\"5000\",\"5000\",\"5000\"]"),
    field("machine_max_acceleration_extruding", "coFloats", "1500,1250", "array", "[\"20000\",\"20000\",\"20000\",\"20000\",\"20000\",\"20000\",\"20000\",\"20000\"]"),
    field("machine_max_acceleration_retracting", "coFloats", "1500,1250", "array", "[\"30000\",\"5000\",\"30000\",\"5000\",\"30000\",\"5000\",\"30000\",\"5000\"]"),
    field("machine_max_acceleration_travel", "coFloats", "0,0", "array", "[\"9000\",\"9000\",\"9000\",\"9000\",\"9000\",\"9000\",\"9000\",\"9000\"]"),
    field("machine_max_acceleration_x", "coFloats", "1000,1000", "array", "[\"20000\",\"20000\",\"20000\",\"20000\",\"20000\",\"20000\",\"20000\",\"20000\"]"),
    field("machine_max_acceleration_y", "coFloats", "1000,1000", "array", "[\"20000\",\"20000\",\"20000\",\"20000\",\"20000\",\"20000\",\"20000\",\"20000\"]"),
    field("machine_max_acceleration_z", "coFloats", "500,200", "array", "[\"500\",\"500\",\"500\",\"500\",\"500\",\"500\",\"500\",\"500\"]"),
    field("machine_max_jerk_e", "coFloats", "2.5,2.5", "array", "[\"2.5\",\"2.5\",\"2.5\",\"2.5\",\"2.5\",\"2.5\",\"2.5\",\"2.5\"]"),
    field("machine_max_jerk_x", "coFloats", "10,10", "array", "[\"9\",\"9\",\"9\",\"9\",\"9\",\"9\",\"9\",\"9\"]"),
    field("machine_max_jerk_y", "coFloats", "10,10", "array", "[\"9\",\"9\",\"9\",\"9\",\"9\",\"9\",\"9\",\"9\"]"),
    field("machine_max_jerk_z", "coFloats", "0.2,0.4", "array", "[\"3\",\"3\",\"3\",\"3\",\"3\",\"3\",\"3\",\"3\"]"),
    field("machine_max_junction_deviation", "coFloats", "0.01", "array", "[\"0.01\"]"),
    field("machine_max_speed_e", "coFloats", "120,120", "array", "[\"30\",\"30\",\"30\",\"30\",\"120\",\"120\",\"120\",\"120\"]"),
    field("machine_max_speed_x", "coFloats", "500,200", "array", "[\"1000\",\"1000\",\"1000\",\"1000\",\"1000\",\"1000\",\"1000\",\"1000\"]"),
    field("machine_max_speed_y", "coFloats", "500,200", "array", "[\"1000\",\"1000\",\"1000\",\"1000\",\"1000\",\"1000\",\"1000\",\"1000\"]"),
    field("machine_max_speed_z", "coFloats", "12,12", "array", "[\"20\",\"20\",\"20\",\"20\",\"20\",\"20\",\"20\",\"20\"]"),
    field("machine_min_extruding_rate", "coFloats", "0,0", "array", "[\"0\",\"0\"]"),
    field("machine_min_travel_rate", "coFloats", "0,0", "array", "[\"0\",\"0\"]"),
    field("max_resonance_avoidance_speed", "coFloat", "120", "scalar_string", "\"120\""),
    field("min_resonance_avoidance_speed", "coFloat", "70", "scalar_string", "\"70\""),
    field("resonance_avoidance", "coBool", "0", "scalar_string", "\"0\""),
];

const DECLARATION_ORDER: [&str; 28] = [
    "emit_machine_limits_to_gcode",
    "machine_max_acceleration_x",
    "machine_max_acceleration_y",
    "machine_max_acceleration_z",
    "machine_max_acceleration_e",
    "machine_max_speed_x",
    "machine_max_speed_y",
    "machine_max_speed_z",
    "machine_max_speed_e",
    "machine_max_acceleration_extruding",
    "machine_max_acceleration_retracting",
    "machine_max_acceleration_travel",
    "machine_max_jerk_x",
    "machine_max_jerk_y",
    "machine_max_jerk_z",
    "machine_max_jerk_e",
    "machine_max_junction_deviation",
    "machine_min_travel_rate",
    "machine_min_extruding_rate",
    "resonance_avoidance",
    "min_resonance_avoidance_speed",
    "max_resonance_avoidance_speed",
    "input_shaping_emit",
    "input_shaping_type",
    "input_shaping_freq_x",
    "input_shaping_freq_y",
    "input_shaping_damp_x",
    "input_shaping_damp_y",
];

const fn field(
    key: &'static str,
    option_type: &'static str,
    default: &'static str,
    wire_shape: &'static str,
    fixture: &'static str,
) -> ExpectedField {
    ExpectedField { key, option_type, default, wire_shape, fixture }
}

#[test]
fn printer_machine_envelope_inventory_and_fixture_are_exact_and_typed() {
    let rows: Vec<InventoryRow> = serde_json::from_str(include_str!(
        "../../../../../tests/ksr_fdmtest_v4/options-v242.json"
    ))
    .unwrap();
    let rows = rows
        .iter()
        .filter(|row| row.static_owner == "machine_envelope_config")
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 28);
    assert_eq!(
        rows.iter().map(|row| row.key.as_str()).collect::<BTreeSet<_>>().len(),
        28
    );
    for (row, expected) in rows.iter().zip(EXPECTED_FIELDS) {
        assert_eq!(row.key, expected.key);
        assert_eq!(row.raw_scope, "printer", "{}", row.key);
        assert_eq!(row.option_type, expected.option_type, "{}", row.key);
        assert_eq!(row.default_serialized, expected.default, "{}", row.key);
        assert_eq!(row.wire_shape, expected.wire_shape, "{}", row.key);
    }

    let fixture = fixture_machine_fields();
    for expected in EXPECTED_FIELDS {
        assert_eq!(
            fixture[expected.key],
            serde_json::from_str::<Value>(expected.fixture).unwrap()
        );
    }
    let parsed: MachineEnvelopeOptions = serde_json::from_value(Value::Object(fixture)).unwrap();
    assert_concrete_types(&parsed);
    let serialized = serde_json::to_string(&parsed).unwrap();
    let canonical: Value = serde_json::from_str(&serialized).unwrap();
    for expected in EXPECTED_FIELDS {
        assert_eq!(
            canonical[expected.key],
            serde_json::from_str::<Value>(expected.fixture).unwrap()
        );
    }
    assert_lexicographic_key_order(&serialized);
}

#[test]
fn printer_machine_envelope_keeps_declaration_and_export_order_separate() {
    assert_eq!(MachineEnvelopeOptions::DECLARATION_ORDER, DECLARATION_ORDER);
    let lexical = EXPECTED_FIELDS.map(|field| field.key);
    assert!(lexical.windows(2).all(|pair| pair[0] < pair[1]));
    assert_ne!(DECLARATION_ORDER, lexical);
    assert_lexicographic_key_order(&serde_json::to_string(&MachineEnvelopeOptions::default()).unwrap());
}

#[test]
fn printer_machine_envelope_defaults_use_fixed_tag_values() {
    let serialized = serde_json::to_value(MachineEnvelopeOptions::default()).unwrap();
    for expected in EXPECTED_FIELDS {
        let expected_value = match expected.option_type {
            "coBool" | "coFloat" => Value::String(expected.default.to_owned()),
            "coEnum" => Value::String(expected.default.to_owned()),
            "coFloats" => Value::Array(
                expected
                    .default
                    .split(',')
                    .map(|value| Value::String(value.to_owned()))
                    .collect(),
            ),
            kind => panic!("unexpected option type {kind}"),
        };
        assert_eq!(serialized[expected.key], expected_value, "{}", expected.key);
    }
}

#[test]
fn printer_machine_envelope_changed_limit_reaches_typed_state() {
    let parsed: PrinterOptions = serde_json::from_str(
        r#"{"machine_max_speed_x":["321","123"],"emit_machine_limits_to_gcode":"0"}"#,
    )
    .unwrap();
    assert_eq!(
        parsed.machine.machine_max_speed_x.0,
        vec![OrcaFloat(321.0), OrcaFloat(123.0)]
    );
    assert_eq!(
        parsed.machine.emit_machine_limits_to_gcode,
        OrcaBool(false)
    );
    assert_eq!(
        parsed.machine.machine_max_speed_y,
        MachineEnvelopeOptions::default().machine_max_speed_y
    );
}

#[test]
fn printer_machine_envelope_rejects_duplicate_unknown_and_invalid_enum_fields() {
    for invalid in [
        r#"{"input_shaping_emit":"0","input_shaping_emit":"1"}"#,
        r#"{"not_a_machine_field":"1"}"#,
        r#"{"input_shaping_type":"TwoHumpEI"}"#,
    ] {
        assert!(serde_json::from_str::<MachineEnvelopeOptions>(invalid).is_err(), "{invalid}");
    }
}

#[test]
fn printer_machine_envelope_input_shaper_type_has_exact_fixed_tag_tokens() {
    for (wire, expected) in [
        ("Default", InputShaperType::Default),
        ("MZV", InputShaperType::Mzv),
        ("ZV", InputShaperType::Zv),
        ("ZVD", InputShaperType::Zvd),
        ("ZVDD", InputShaperType::Zvdd),
        ("ZVDDD", InputShaperType::Zvddd),
        ("EI", InputShaperType::Ei),
        ("EI2", InputShaperType::Ei2),
        ("2HUMP_EI", InputShaperType::TwoHumpEi),
        ("EI3", InputShaperType::Ei3),
        ("3HUMP_EI", InputShaperType::ThreeHumpEi),
        ("DAA", InputShaperType::Daa),
        ("Disable", InputShaperType::Disable),
    ] {
        let parsed: InputShaperType = serde_json::from_str(&format!(r#""{wire}""#)).unwrap();
        assert_eq!(parsed, expected);
        assert_eq!(serde_json::to_string(&parsed).unwrap(), format!(r#""{wire}""#));
    }
    for invalid in ["default", "mzv", "TwoHumpEI", "2HUMP-EI", "unknown"] {
        assert!(
            serde_json::from_str::<InputShaperType>(&format!(r#""{invalid}""#)).is_err(),
            "{invalid}"
        );
    }
}

#[test]
fn project_settings_starts_with_only_the_typed_printer_machine_child() {
    let expected = MachineEnvelopeOptions::default();
    assert_eq!(PrinterOptions::default().machine, expected);
    assert_eq!(ProjectSettings::default().printer.machine, expected);
}

fn fixture_machine_fields() -> Map<String, Value> {
    let fixture = super::project_fixture::project_settings_value();
    let fixture = fixture.as_object().unwrap();
    EXPECTED_FIELDS
        .iter()
        .map(|expected| (expected.key.to_owned(), fixture[expected.key].clone()))
        .collect()
}

fn assert_lexicographic_key_order(serialized: &str) {
    let mut previous = 0;
    for expected in EXPECTED_FIELDS {
        let marker = format!("\"{}\":", expected.key);
        let position = serialized.find(&marker).unwrap();
        assert!(position >= previous, "{} is out of order", expected.key);
        previous = position;
    }
}

fn assert_concrete_types(value: &MachineEnvelopeOptions) {
    let _: &OrcaBool = &value.emit_machine_limits_to_gcode;
    let _: &OrcaFloats = &value.machine_max_acceleration_x;
    let _: &OrcaFloats = &value.machine_max_acceleration_y;
    let _: &OrcaFloats = &value.machine_max_acceleration_z;
    let _: &OrcaFloats = &value.machine_max_acceleration_e;
    let _: &OrcaFloats = &value.machine_max_speed_x;
    let _: &OrcaFloats = &value.machine_max_speed_y;
    let _: &OrcaFloats = &value.machine_max_speed_z;
    let _: &OrcaFloats = &value.machine_max_speed_e;
    let _: &OrcaFloats = &value.machine_max_acceleration_extruding;
    let _: &OrcaFloats = &value.machine_max_acceleration_retracting;
    let _: &OrcaFloats = &value.machine_max_acceleration_travel;
    let _: &OrcaFloats = &value.machine_max_jerk_x;
    let _: &OrcaFloats = &value.machine_max_jerk_y;
    let _: &OrcaFloats = &value.machine_max_jerk_z;
    let _: &OrcaFloats = &value.machine_max_jerk_e;
    let _: &OrcaFloats = &value.machine_max_junction_deviation;
    let _: &OrcaFloats = &value.machine_min_travel_rate;
    let _: &OrcaFloats = &value.machine_min_extruding_rate;
    let _: &OrcaBool = &value.resonance_avoidance;
    let _: &OrcaFloat = &value.min_resonance_avoidance_speed;
    let _: &OrcaFloat = &value.max_resonance_avoidance_speed;
    let _: &OrcaBool = &value.input_shaping_emit;
    let _: &InputShaperType = &value.input_shaping_type;
    let _: &OrcaFloat = &value.input_shaping_freq_x;
    let _: &OrcaFloat = &value.input_shaping_freq_y;
    let _: &OrcaFloat = &value.input_shaping_damp_x;
    let _: &OrcaFloat = &value.input_shaping_damp_y;
}
