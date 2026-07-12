use super::super::*;
use crate::{BrimType, PrintPathRole, SliceError};
use serde_json::{Map, Value, json};

#[test]
fn preserves_unknown_orca_keys() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "wall_loops": 2,
        "filament_colour": ["#FFFFFF"]
    }))
    .unwrap();

    assert_eq!(options.values().len(), 3);
    assert_eq!(options.values()["layer_height"], json!(0.2));
    assert_eq!(options.values()["wall_loops"], json!(2));
    assert_eq!(options.values()["filament_colour"], json!(["#FFFFFF"]));
}

#[test]
fn hardware_options_use_orca_defaults() {
    let options = SliceOptions::default();

    let hardware = options.hardware_options().unwrap();

    assert_eq!(hardware.nozzle_diameters(), &[0.4]);
    assert_eq!(hardware.filament_diameters(), &[1.75]);
    assert_eq!(hardware.min_layer_heights(), &[0.07]);
    assert_eq!(hardware.max_layer_heights(), &[0.0]);
}

#[test]
fn parses_orca_numeric_vector_forms() {
    let options: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": ["0.4", "0.6"],
        "filament_diameter": "1.75;2.85",
        "min_layer_height": "0.06,0.08",
        "max_layer_height": [0.28, 0.42]
    }))
    .unwrap();

    assert_eq!(options.nozzle_diameters().unwrap(), vec![0.4, 0.6]);
    assert_eq!(options.filament_diameters().unwrap(), vec![1.75, 2.85]);
    assert_eq!(options.min_layer_heights().unwrap(), vec![0.06, 0.08]);
    assert_eq!(options.max_layer_heights().unwrap(), vec![0.28, 0.42]);
}

#[test]
fn parses_scalar_number_and_string_as_single_value_vectors() {
    let options: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": 0.4,
        "filament_diameter": "1.75",
        "min_layer_height": 0.07,
        "max_layer_height": "0"
    }))
    .unwrap();

    assert_eq!(options.nozzle_diameters().unwrap(), vec![0.4]);
    assert_eq!(options.filament_diameters().unwrap(), vec![1.75]);
    assert_eq!(options.min_layer_heights().unwrap(), vec![0.07]);
    assert_eq!(options.max_layer_heights().unwrap(), vec![0.0]);
}

#[test]
fn rejects_invalid_hardware_option_values() {
    for (key, value) in [
        ("nozzle_diameter", json!(0.004)),
        ("filament_diameter", json!(0.99)),
        ("min_layer_height", json!(-0.01)),
        ("max_layer_height", json!(-0.01)),
        ("nozzle_diameter", json!("nil")),
        ("nozzle_diameter", json!("")),
        ("nozzle_diameter", json!([])),
        ("nozzle_diameter", json!(["0.4", "bad"])),
        ("nozzle_diameter", json!([["0.4"]])),
        ("nozzle_diameter", json!({"value": "0.4"})),
        ("nozzle_diameter", json!(true)),
        ("nozzle_diameter", Value::Null),
    ] {
        let mut values = Map::new();
        values.insert(key.to_owned(), value);
        let options: SliceOptions = serde_json::from_value(Value::Object(values)).unwrap();

        let err = options.hardware_options().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
    }
}

#[test]
fn infill_options_use_orca_defaults_and_nozzle_width() {
    let infill = SliceOptions::default().infill_options().unwrap();
    assert_eq!(infill.sparse_density_percent(), 20.0);
    assert_eq!(infill.direction_degrees(), 45.0);
    assert_eq!(infill.line_width(), 0.4);
    assert_eq!(infill.pattern(), InfillPattern::CrossHatch);
}

#[test]
fn parses_sparse_infill_options() {
    let options: SliceOptions = serde_json::from_value(json!({
        "sparse_infill_density": "50", "infill_direction": 0,
        "sparse_infill_line_width": 0.5
    }))
    .unwrap();
    let infill = options.infill_options().unwrap();
    assert_eq!(infill.sparse_density_percent(), 50.0);
    assert_eq!(infill.direction_degrees(), 0.0);
    assert_eq!(infill.line_width(), 0.5);
}

#[test]
fn parses_percent_sparse_infill_line_width_for_pipeline() {
    let options: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": [0.4],
        "sparse_infill_line_width": "120%"
    }))
    .unwrap();

    let infill = options.infill_options().unwrap();

    assert_eq!(infill.line_width(), 0.48);
}

#[test]
fn rejects_invalid_sparse_infill_options() {
    for value in [json!(-1), json!(101), json!(null), json!("bad")] {
        let options: SliceOptions =
            serde_json::from_value(json!({"sparse_infill_density": value})).unwrap();
        assert!(is_invalid_input(options.infill_options()));
    }
    let options: SliceOptions = serde_json::from_value(json!({"infill_direction": 361})).unwrap();
    assert!(is_invalid_input(options.infill_options()));
    let options: SliceOptions =
        serde_json::from_value(json!({"sparse_infill_line_width": -0.1})).unwrap();
    assert!(is_invalid_input(options.infill_options()));
}

#[test]
fn parses_extrusion_width_options_for_e_values() {
    let options: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": [0.4],
        "filament_diameter": [2.0],
        "line_width": "120%",
        "outer_wall_line_width": 0,
        "sparse_infill_line_width": "0.3"
    }))
    .unwrap();

    let extrusion = options.extrusion_options().unwrap();

    assert_eq!(extrusion.filament_diameter(), 2.0);
    assert_eq!(
        extrusion.width_for_role(PrintPathRole::ExternalPerimeter),
        0.48
    );
    assert_eq!(extrusion.width_for_role(PrintPathRole::SparseInfill), 0.3);
}

#[test]
fn extrusion_width_options_use_orca_automatic_width_when_zero() {
    let options: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": [0.4],
        "filament_diameter": [2.0],
        "line_width": 0,
        "outer_wall_line_width": 0,
        "sparse_infill_line_width": 0
    }))
    .unwrap();

    let extrusion = options.extrusion_options().unwrap();

    assert_eq!(
        extrusion.width_for_role(PrintPathRole::ExternalPerimeter),
        0.45
    );
    assert_eq!(extrusion.width_for_role(PrintPathRole::SparseInfill), 0.45);
}

#[test]
fn rejects_invalid_extrusion_width_options() {
    let options: SliceOptions = serde_json::from_value(json!({ "line_width": "abc%" })).unwrap();
    assert!(matches!(
        options.extrusion_options(),
        Err(SliceError::InvalidInput(_))
    ));

    let options: SliceOptions =
        serde_json::from_value(json!({ "outer_wall_line_width": -0.1 })).unwrap();
    assert!(matches!(
        options.extrusion_options(),
        Err(SliceError::InvalidInput(_))
    ));
}

#[test]
fn brim_options_use_orca_defaults() {
    let brim = SliceOptions::default().brim_options().unwrap();
    assert_eq!(brim.width_mm(), 0.0);
    assert_eq!(brim.object_gap_mm(), 0.0);
    assert_eq!(brim.brim_type(), BrimType::AutoBrim);
}

#[test]
fn parses_brim_options() {
    let options: SliceOptions = serde_json::from_value(json!({
        "brim_width": "1.2",
        "brim_object_gap": "0.2",
        "brim_type": "outer_only"
    }))
    .unwrap();
    let brim = options.brim_options().unwrap();
    assert_eq!(brim.width_mm(), 1.2);
    assert_eq!(brim.object_gap_mm(), 0.2);
    assert_eq!(brim.brim_type(), BrimType::OuterOnly);
}

#[test]
fn rejects_invalid_brim_options() {
    for value in [
        json!({"brim_width": -0.1}),
        json!({"brim_width": 101}),
        json!({"brim_width": "wide"}),
        json!({"brim_object_gap": -0.1}),
        json!({"brim_object_gap": 2.1}),
        json!({"brim_type": "outer"}),
        json!({"brim_type": 1}),
    ] {
        let options: SliceOptions = serde_json::from_value(value).unwrap();
        assert!(options.brim_options().is_err());
    }
}

#[test]
fn skirt_options_use_orca_defaults() {
    let options = SliceOptions::default().skirt_options().unwrap();
    assert_eq!(options.loops(), 1);
    assert_eq!(options.distance_mm(), 2.0);
    assert_eq!(options.height_layers(), 1);
    assert_eq!(options.speed_mm_s(), 50.0);
}

#[test]
fn parses_skirt_options() {
    let options: SliceOptions = serde_json::from_value(json!({
        "skirt_loops": "2",
        "skirt_distance": "3.5",
        "skirt_height": "2",
        "skirt_speed": "0"
    }))
    .unwrap();
    let skirt = options.skirt_options().unwrap();
    assert_eq!(skirt.loops(), 2);
    assert_eq!(skirt.distance_mm(), 3.5);
    assert_eq!(skirt.height_layers(), 2);
    assert_eq!(skirt.speed_mm_s(), 0.0);
}

#[test]
fn rejects_invalid_skirt_options() {
    for value in [
        json!({"skirt_loops": -1}),
        json!({"skirt_distance": -0.1}),
        json!({"skirt_loops": 1.5}),
        json!({"skirt_height": -1}),
        json!({"skirt_height": 1.5}),
        json!({"skirt_speed": -1}),
        json!({"skirt_speed": "fast"}),
    ] {
        let options: SliceOptions = serde_json::from_value(value).unwrap();
        assert!(options.skirt_options().is_err());
    }
}

#[test]
fn bridge_options_use_orca_defaults() {
    let bridge = SliceOptions::default().bridge_options().unwrap();
    assert_eq!(bridge.bridge_flow(), 1.0);
    assert_eq!(bridge.internal_bridge_flow(), 1.0);
    assert_eq!(bridge.bridge_speed_mm_s(), 25.0);
    assert_eq!(bridge.internal_bridge_speed_mm_s(), 37.5);
    assert!(!bridge.bridge_no_support());
    assert!(!bridge.thick_bridges());
}

#[test]
fn parses_bridge_options() {
    let options: SliceOptions = serde_json::from_value(json!({
        "bridge_flow": "0.9",
        "internal_bridge_flow": 0.8,
        "bridge_speed": "30",
        "internal_bridge_speed": "200%",
        "bridge_no_support": true,
        "thick_bridges": true
    }))
    .unwrap();
    let bridge = options.bridge_options().unwrap();
    assert_eq!(bridge.bridge_flow(), 0.9);
    assert_eq!(bridge.internal_bridge_flow(), 0.8);
    assert_eq!(bridge.bridge_speed_mm_s(), 30.0);
    assert_eq!(bridge.internal_bridge_speed_mm_s(), 60.0);
    assert!(bridge.bridge_no_support());
    assert!(bridge.thick_bridges());
}

#[test]
fn rejects_invalid_bridge_options() {
    for value in [
        json!({"bridge_flow": 0}),
        json!({"bridge_flow": 2.1}),
        json!({"internal_bridge_flow": -0.1}),
        json!({"bridge_speed": 0}),
        json!({"internal_bridge_speed": "0%"}),
        json!({"internal_bridge_speed": "fast"}),
        json!({"bridge_no_support": "yes"}),
        json!({"thick_bridges": 1}),
    ] {
        let options: SliceOptions = serde_json::from_value(value).unwrap();
        assert!(options.bridge_options().is_err());
    }
}

#[test]
fn default_internal_bridge_speed_uses_resolved_bridge_speed() {
    let options: SliceOptions = serde_json::from_value(json!({"bridge_speed": 40})).unwrap();

    let bridge = options.bridge_options().unwrap();

    assert_eq!(bridge.bridge_speed_mm_s(), 40.0);
    assert_eq!(bridge.internal_bridge_speed_mm_s(), 60.0);
}

#[test]
fn speed_options_use_orca_defaults() {
    let speeds = SliceOptions::default().speed_options().unwrap();
    assert_eq!(speeds.travel_speed_mm_s(), 120.0);
    assert_eq!(speeds.external_perimeter_speed_mm_s(), 60.0);
    assert_eq!(speeds.sparse_infill_speed_mm_s(), 100.0);
}

#[test]
fn parses_speed_options() {
    let options: SliceOptions = serde_json::from_value(json!({
        "travel_speed": "150",
        "outer_wall_speed": 45,
        "sparse_infill_speed": "80"
    }))
    .unwrap();
    let speeds = options.speed_options().unwrap();
    assert_eq!(speeds.travel_speed_mm_s(), 150.0);
    assert_eq!(speeds.external_perimeter_speed_mm_s(), 45.0);
    assert_eq!(speeds.sparse_infill_speed_mm_s(), 80.0);
}

#[test]
fn rejects_invalid_speed_options() {
    for (key, value) in [
        ("travel_speed", json!(0)),
        ("outer_wall_speed", json!(-1)),
        ("sparse_infill_speed", json!("fast")),
        ("travel_speed", json!(null)),
    ] {
        let mut values = Map::new();
        values.insert(key.to_owned(), value);
        let options: SliceOptions = serde_json::from_value(Value::Object(values)).unwrap();
        assert!(matches!(
            options.speed_options(),
            Err(SliceError::InvalidInput(_))
        ));
    }
}

#[test]
fn infill_first_uses_orca_default_false() {
    assert!(!SliceOptions::default().is_infill_first().unwrap());
}

#[test]
fn parses_infill_first_boolean() {
    let options: SliceOptions = serde_json::from_value(json!({"is_infill_first": true})).unwrap();
    assert!(options.is_infill_first().unwrap());
}

#[test]
fn rejects_non_boolean_infill_first() {
    for value in [json!(1), json!("true"), json!(null)] {
        let options: SliceOptions =
            serde_json::from_value(json!({"is_infill_first": value})).unwrap();
        assert!(matches!(
            options.is_infill_first(),
            Err(SliceError::InvalidInput(_))
        ));
    }
}

fn is_invalid_input(result: Result<InfillOptions, SliceError>) -> bool {
    matches!(result, Err(SliceError::InvalidInput(_)))
}
