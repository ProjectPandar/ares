use serde_json::{Map, Value};

use super::{ProcessObjectSourceOptions, ProcessOptions, inventory, object_rows};

#[test]
fn process_object_source_every_field_accepts_nondefault_typed_state() {
    let rows = inventory();
    for row in object_rows(&rows) {
        let alternate = alternate(row.key.as_str(), &row.option_type, &row.default_serialized);
        assert_ne!(alternate, row.default_serialized, "{}", row.key);
        let input = Map::from_iter([(row.key.clone(), Value::String(alternate.clone()))]);
        let value: ProcessObjectSourceOptions =
            serde_json::from_value(Value::Object(input)).unwrap();
        assert_eq!(serde_json::to_value(value).unwrap()[&row.key], alternate, "{}", row.key);
    }
}

#[test]
fn process_object_source_rejects_duplicate_unknown_wrong_shape_and_deferred_scopes() {
    for invalid in [
        r#"{"layer_height":"0.2","layer_height":"0.3"}"#,
        r#"{"unknown_object_option":"1"}"#,
        r#"{"layer_height":[]}"#,
        r#"{"enable_support":null}"#,
        r#"{"raft_layers":{}}"#,
        r#"{"wall_transition_length":[]}"#,
        r#"{"brim_type":[]}"#,
        r#"{"initial_layer_print_height":"0.2"}"#,
        r#"{"independent_support_layer_height":"1"}"#,
        r#"{"printable_height":"256"}"#,
        r#"{"filament_type":["PLA"]}"#,
        r#"{"adaptive_layer_height":"1"}"#,
    ] {
        assert!(
            serde_json::from_str::<ProcessObjectSourceOptions>(invalid).is_err(),
            "{invalid}"
        );
        assert!(serde_json::from_str::<ProcessOptions>(invalid).is_err(), "{invalid}");
    }
    for region in [
        r#"{"wall_loops":"2"}"#,
        r#"{"sparse_infill_density":"20%"}"#,
    ] {
        assert!(
            serde_json::from_str::<ProcessObjectSourceOptions>(region).is_err(),
            "{region}"
        );
        assert!(serde_json::from_str::<ProcessOptions>(region).is_ok(), "{region}");
    }
}

#[test]
fn process_object_source_duplicate_and_unknown_errors_stay_specific() {
    let duplicate = serde_json::from_str::<ProcessObjectSourceOptions>(
        r#"{"layer_height":"0.2","layer_height":"0.3"}"#,
    )
    .unwrap_err()
    .to_string();
    assert!(duplicate.contains("duplicate Orca option layer_height"), "{duplicate}");

    let unknown = serde_json::from_str::<ProcessObjectSourceOptions>(
        r#"{"unknown_object_option":"1"}"#,
    )
    .unwrap_err()
    .to_string();
    assert!(unknown.contains("unknown Orca process object option unknown_object_option"), "{unknown}");
}

fn alternate(key: &str, option_type: &str, default: &str) -> String {
    match option_type {
        "coBool" => if default == "0" { "1" } else { "0" }.to_owned(),
        "coFloat" => "7.125".to_owned(),
        "coInt" => "7".to_owned(),
        "coPercent" => "37%".to_owned(),
        "coFloatOrPercent" => match key {
            "bridge_acceleration" | "sparse_infill_acceleration" | "support_threshold_overlap" => {
                "7.125"
            }
            "internal_solid_infill_acceleration" | "line_width" | "support_line_width" => "37%",
            _ => unreachable!("unexpected float-or-percent {key}"),
        }
        .to_owned(),
        "coEnum" => match key {
            "brim_type" => "no_brim",
            "dont_filter_internal_bridges" => "limited",
            "enable_extra_bridge_layer" => "external_bridge_only",
            "gap_fill_target" => "everywhere",
            "seam_position" => "random",
            "slicing_mode" => "close_holes",
            "support_base_pattern" => "hollow",
            "support_interface_pattern" => "grid",
            "support_ironing_pattern" => "concentric",
            "support_style" => "organic",
            "support_type" => "tree(auto)",
            "wall_generator" => "classic",
            _ => unreachable!("unexpected enum {key}"),
        }
        .to_owned(),
        _ => unreachable!("unexpected option type {option_type}"),
    }
}
