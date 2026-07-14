mod expected;
mod enums;
mod type_assertions;

use std::collections::BTreeSet;

use serde::Deserialize;
use serde_json::{Map, Value};

use super::super::{
    GCodeFlavor, Nullable, NozzleType, OrcaBool, OrcaFloat, OrcaInt,
    PrinterGCodeSourceOptions, PrinterOptions,
};
use expected::{DECLARATION_ORDER, EXPECTED_FIELDS};

#[derive(Debug, Deserialize)]
struct InventoryRow {
    key: String,
    raw_scope: String,
    static_owner: String,
    option_type: String,
    default_serialized: String,
    wire_shape: String,
    nullable: bool,
}

#[test]
fn printer_gcode_source_inventory_is_exact_disjoint_and_typed() {
    let rows = inventory_rows();
    assert_eq!(rows.len(), 62);
    assert_eq!(
        rows.iter().map(|row| row.key.as_str()).collect::<BTreeSet<_>>().len(),
        62
    );
    for (row, expected) in rows.iter().zip(EXPECTED_FIELDS) {
        assert_eq!(row.key, expected.key);
        assert_eq!(row.raw_scope, "printer", "{}", row.key);
        assert_eq!(row.static_owner, "g_code_config", "{}", row.key);
        assert_eq!(row.option_type, expected.option_type, "{}", row.key);
        assert_eq!(row.default_serialized, expected.default, "{}", row.key);
        assert_eq!(row.wire_shape, expected.wire_shape, "{}", row.key);
        assert_eq!(row.nullable, expected.nullable, "{}", row.key);
        assert!(
            !super::super::MachineEnvelopeOptions::DECLARATION_ORDER.contains(&row.key.as_str()),
            "{} overlaps MachineEnvelopeOptions",
            row.key
        );
    }
}

#[test]
fn printer_gcode_source_fixture_round_trips_every_field_byte_for_byte() {
    let fixture = fixture_gcode_fields();
    let parsed: PrinterGCodeSourceOptions =
        serde_json::from_value(Value::Object(fixture.clone())).unwrap();
    type_assertions::assert_concrete_types(&parsed);
    let serialized = serde_json::to_string(&parsed).unwrap();
    let canonical: Value = serde_json::from_str(&serialized).unwrap();
    assert_eq!(canonical, Value::Object(fixture));
    assert_lexicographic_key_order(&serialized);
}

#[test]
fn printer_gcode_source_defaults_match_fixed_tag_definitions() {
    let defaults = serde_json::to_value(PrinterGCodeSourceOptions::default()).unwrap();
    for expected in EXPECTED_FIELDS {
        let value: Value = serde_json::from_str(expected.semantic_default_json).unwrap();
        assert_eq!(defaults[expected.key], value, "{}", expected.key);
    }
    assert_eq!(defaults["machine_end_gcode"].as_str().unwrap().chars().filter(|c| *c == '\n').count(), 3);
    assert_eq!(defaults["machine_start_gcode"].as_str().unwrap().chars().filter(|c| *c == '\n').count(), 2);
    assert_eq!(defaults["printer_extruder_variant"], serde_json::json!(["Direct Drive Standard"]));
    assert_eq!(defaults["wrapping_exclude_area"], serde_json::json!([]));
}

#[test]
fn printer_gcode_source_preserves_upstream_declaration_order_separately() {
    assert_eq!(PrinterGCodeSourceOptions::DECLARATION_ORDER, DECLARATION_ORDER);
    let lexical = EXPECTED_FIELDS.map(|field| field.key);
    assert!(lexical.windows(2).all(|pair| pair[0] < pair[1]));
    assert_ne!(DECLARATION_ORDER, lexical);
}

#[test]
fn printer_gcode_source_flat_dispatch_keeps_machine_and_gcode_children_independent() {
    let parsed: PrinterOptions = serde_json::from_str(
        r#"{"machine_max_speed_x":["321","123"],"auxiliary_fan":"1","gcode_flavor":"klipper"}"#,
    )
    .unwrap();
    assert_eq!(parsed.machine.machine_max_speed_x.0[0], OrcaFloat(321.0));
    assert_eq!(parsed.gcode.auxiliary_fan, OrcaBool(true));
    assert_eq!(parsed.gcode.gcode_flavor, GCodeFlavor::Klipper);
    assert_eq!(
        parsed.machine.machine_max_speed_y,
        super::super::MachineEnvelopeOptions::default().machine_max_speed_y
    );
}

#[test]
fn printer_gcode_source_preserves_vector_cardinality_and_multiline_gcode() {
    let parsed: PrinterGCodeSourceOptions =
        serde_json::from_value(Value::Object(fixture_gcode_fields())).unwrap();
    assert_eq!(parsed.extruder_type.0.len(), 2);
    assert_eq!(parsed.nozzle_flush_dataset.0.len(), 4);
    assert_eq!(parsed.nozzle_type.0.len(), 4);
    assert_eq!(parsed.physical_extruder_map.0.len(), 2);
    assert_eq!(parsed.printer_extruder_id.0.len(), 4);
    assert_eq!(parsed.printer_extruder_variant.0.len(), 4);
    assert_eq!(parsed.retract_lift_enforce.0.len(), 4);
    assert_eq!(parsed.retraction_distances_when_cut.0.len(), 4);
    assert_eq!(parsed.travel_slope.0.len(), 4);
    assert_eq!(parsed.z_hop_types.0.len(), 4);
    assert!(parsed.machine_start_gcode.0.contains('\n'));
    assert!(parsed.machine_end_gcode.0.contains('\n'));
    assert!(parsed.change_filament_gcode.0.contains('\n'));
    assert_eq!(
        serde_json::to_value(&parsed).unwrap()["machine_start_gcode"],
        fixture_gcode_fields()["machine_start_gcode"]
    );
}

#[test]
fn printer_gcode_source_nullable_vectors_are_element_nullable() {
    let parsed: PrinterGCodeSourceOptions = serde_json::from_str(
        r#"{"nozzle_flush_dataset":["nil","7"],"nozzle_type":["nil","brass"]}"#,
    )
    .unwrap();
    assert_eq!(
        parsed.nozzle_flush_dataset.0,
        vec![Nullable::Nil, Nullable::Value(OrcaInt(7))]
    );
    assert_eq!(
        parsed.nozzle_type.0,
        vec![Nullable::Nil, Nullable::Value(NozzleType::Brass)]
    );
    let wire = serde_json::to_value(parsed).unwrap();
    assert_eq!(wire["nozzle_flush_dataset"], serde_json::json!(["nil", "7"]));
    assert_eq!(wire["nozzle_type"], serde_json::json!(["nil", "brass"]));

    let all_nil: PrinterGCodeSourceOptions = serde_json::from_str(
        r#"{"nozzle_flush_dataset":["nil","nil"],"nozzle_type":["nil","nil"]}"#,
    )
    .unwrap();
    let wire = serde_json::to_value(all_nil).unwrap();
    assert_eq!(wire["nozzle_flush_dataset"], serde_json::json!(["nil", "nil"]));
    assert_eq!(wire["nozzle_type"], serde_json::json!(["nil", "nil"]));
}

#[test]
fn printer_gcode_source_points_use_the_upstream_co_points_wire_shape() {
    let parsed: PrinterGCodeSourceOptions =
        serde_json::from_str(r#"{"wrapping_exclude_area":["1x2","-3.5x4"]}"#).unwrap();
    assert_eq!(
        parsed.wrapping_exclude_area.0,
        vec![
            super::super::Point2d::new(1.0, 2.0),
            super::super::Point2d::new(-3.5, 4.0),
        ]
    );
    assert_eq!(
        serde_json::to_value(parsed).unwrap()["wrapping_exclude_area"],
        serde_json::json!(["1x2", "-3.5x4"])
    );
}

#[test]
fn printer_gcode_source_preserves_all_five_long_fixture_gcodes_exactly() {
    let fixture = fixture_gcode_fields();
    let parsed: PrinterGCodeSourceOptions =
        serde_json::from_value(Value::Object(fixture.clone())).unwrap();
    for (key, actual) in [
        ("change_filament_gcode", &parsed.change_filament_gcode.0),
        ("layer_change_gcode", &parsed.layer_change_gcode.0),
        ("machine_end_gcode", &parsed.machine_end_gcode.0),
        ("machine_start_gcode", &parsed.machine_start_gcode.0),
        ("time_lapse_gcode", &parsed.time_lapse_gcode.0),
    ] {
        let expected = fixture[key].as_str().unwrap();
        assert_eq!(actual, expected, "{key}");
        assert_eq!(actual.len(), expected.len(), "{key} byte length");
        assert_eq!(
            actual.bytes().filter(|byte| *byte == b'\n').count(),
            expected.bytes().filter(|byte| *byte == b'\n').count(),
            "{key} LF count"
        );
        assert_eq!(actual.ends_with('\n'), expected.ends_with('\n'), "{key} trailing LF");
    }
}

#[test]
fn printer_gcode_source_rejects_duplicate_unknown_and_invalid_enum_fields() {
    for invalid in [
        r#"{"auxiliary_fan":"0","auxiliary_fan":"1"}"#,
        r#"{"not_a_printer_gcode_field":"1"}"#,
        r#"{"bed_temperature_formula":"highest"}"#,
        r#"{"enable_power_loss_recovery":"Printer configuration"}"#,
        r#"{"extruder_type":["direct drive"]}"#,
        r#"{"gcode_flavor":"Marlin"}"#,
        r#"{"nozzle_type":["E3D"]}"#,
        r#"{"printer_structure":"CoreXY"}"#,
        r#"{"retract_lift_enforce":["all surfaces"]}"#,
        r#"{"wipe_tower_type":"Type2"}"#,
        r#"{"z_hop_types":["Slope"]}"#,
        r#"{"extruder_printable_area":[]}"#,
        r#"{"extruder_offset":[]}"#,
        r#"{"bed_shape":[]}"#,
    ] {
        assert!(
            serde_json::from_str::<PrinterGCodeSourceOptions>(invalid).is_err(),
            "{invalid}"
        );
    }
}

fn inventory_rows() -> Vec<InventoryRow> {
    serde_json::from_str::<Vec<InventoryRow>>(include_str!(
        "../../../../../tests/ksr_fdmtest_v4/options-v242.json"
    ))
    .unwrap()
    .into_iter()
    .filter(|row| row.raw_scope == "printer" && row.static_owner == "g_code_config")
    .collect()
}

fn fixture_gcode_fields() -> Map<String, Value> {
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
