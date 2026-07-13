use std::collections::BTreeSet;

use serde_json::{Map, Value, json};

use super::direct_dispatch::alternate;
use super::expected::{
    Child, GCODE_DECLARATION_ORDER, PRESET_DECLARATION_ORDER, PRINT_DECLARATION_ORDER,
    PRODUCTION_LITERAL_COMPLEMENT, REAL_FIELDS,
};
use super::super::super::{
    ProjectGCodeSourceOptions, ProjectPrintSourceOptions, ProjectPresetSourceOptions,
    ProjectRuntimeOptions,
};
use super::{child_output, expected_defaults, parent_output, serialized_key_order};

#[test]
fn standalone_children_and_flat_parent_emit_exact_lexical_default_bytes() {
    for (child, serialized) in [
        (
            Child::GCode,
            serde_json::to_string(&ProjectGCodeSourceOptions::default()).unwrap(),
        ),
        (
            Child::Print,
            serde_json::to_string(&ProjectPrintSourceOptions::default()).unwrap(),
        ),
        (
            Child::Preset,
            serde_json::to_string(&ProjectPresetSourceOptions::default()).unwrap(),
        ),
    ] {
        let expected = expected_defaults(Some(child));
        assert_eq!(
            serialized_key_order(&serialized),
            expected.keys().cloned().collect::<Vec<_>>()
        );
        assert_eq!(serialized.as_bytes(), serde_json::to_vec(&expected).unwrap());
    }

    let expected = expected_defaults(None);
    let serialized = serde_json::to_string(&ProjectRuntimeOptions::default()).unwrap();
    assert_eq!(serialized_key_order(&serialized), expected.keys().cloned().collect::<Vec<_>>());
    assert_eq!(serialized.as_bytes(), serde_json::to_vec(&expected).unwrap());
    for nested in ["gcode", "print", "preset"] {
        assert!(!serialized.contains(&format!("\"{nested}\":{{")));
    }

    assert_ne!(GCODE_DECLARATION_ORDER.as_slice(), child_lexical_keys(Child::GCode).as_slice());
    assert_ne!(PRINT_DECLARATION_ORDER.as_slice(), child_lexical_keys(Child::Print).as_slice());
    assert_ne!(PRESET_DECLARATION_ORDER.as_slice(), child_lexical_keys(Child::Preset).as_slice());
}

#[test]
fn every_vector_accepts_empty_one_and_three_valid_elements_without_cardinality_rules() {
    for field in REAL_FIELDS.iter().filter(|field| field.is_array) {
        let alternate = alternate(field);
        let element = alternate.as_array().unwrap()[0].clone();
        for length in [0, 1, 3] {
            let values = Value::Array(vec![element.clone(); length]);
            let input = Value::Object(Map::from_iter([(
                field.key.to_owned(),
                values.clone(),
            )]));
            assert_eq!(child_output(field, input.clone())[field.key], values, "{}", field.key);
            assert_eq!(parent_output(input)[field.key], values, "{}", field.key);
        }
    }
}

#[test]
fn opaque_ams_empty_string_points_percent_bool_and_finite_numbers_remain_distinct() {
    for (key, value) in [
        ("extruder_ams_count", json!(["1#0|4#0", "", "raw#payload|kept"])),
        ("filament_ids", json!([])),
        ("extruder_colour", json!([""])),
        ("extruder_offset", json!(["7x8", "-9.5x10", "0x0"])),
        ("retract_before_wipe", json!(["37%", "0%", "100%"])),
        ("wipe", json!(["1", "0", "1"])),
        ("flush_volumes_matrix", json!(["0", "7.125", "-9.5"])),
    ] {
        let field = REAL_FIELDS.iter().find(|field| field.key == key).unwrap();
        let input = Value::Object(Map::from_iter([(key.to_owned(), value.clone())]));
        assert_eq!(child_output(field, input.clone())[key], value);
        assert_eq!(parent_output(input)[key], value);
    }
    let empty = parent_output(json!({"filament_ids": []}));
    let one_empty = parent_output(json!({"filament_ids": [""]}));
    assert_ne!(empty["filament_ids"], one_empty["filament_ids"]);
}

#[test]
fn dynamic_collision_complement_is_exact_without_migrating_any_consumer() {
    let all = REAL_FIELDS.iter().map(|field| field.key).collect::<BTreeSet<_>>();
    let complement = BTreeSet::from(PRODUCTION_LITERAL_COMPLEMENT);
    assert_eq!(complement.len(), 13);
    assert!(complement.is_subset(&all));
    assert_eq!(all.difference(&complement).count(), 31);
}

fn child_lexical_keys(child: Child) -> Vec<&'static str> {
    REAL_FIELDS
        .iter()
        .filter(|field| field.child == child)
        .map(|field| field.key)
        .collect()
}
