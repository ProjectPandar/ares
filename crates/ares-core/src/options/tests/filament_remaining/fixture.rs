use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use super::super::super::{
    FilamentOptions, FilamentPrintSourceOptions, FilamentRegionSourceOptions,
    FilamentRetractOverrideOptions,
};
use super::expected::{FIXTURE_OVERRIDE_KEYS, VARIANT_KEYS};
use super::{expected_default, fixture_fields, inventory, remaining_rows};

#[test]
fn fixture_preserves_each_child_and_complete_flat_parent_bytes() {
    let rows = inventory();
    let remaining = remaining_rows(&rows);
    for owner in ["print_config", "print_region_config"] {
        let keys = remaining
            .iter()
            .filter(|row| row.static_owner == owner)
            .map(|row| row.key.as_str());
        let fixture = fixture_fields(keys);
        let serialized = match owner {
            "print_config" => serde_json::to_value(
                serde_json::from_value::<FilamentPrintSourceOptions>(Value::Object(fixture.clone()))
                    .unwrap(),
            )
            .unwrap(),
            "print_region_config" => serde_json::to_value(
                serde_json::from_value::<FilamentRegionSourceOptions>(Value::Object(
                    fixture.clone(),
                ))
                .unwrap(),
            )
            .unwrap(),
            _ => unreachable!(),
        };
        assert_eq!(serialized, Value::Object(fixture), "{owner}");
    }
    let retract_keys = remaining
        .iter()
        .filter(|row| row.static_owner == "unowned" && row.key != "pellet_flow_coefficient")
        .map(|row| row.key.as_str());
    let retract_fixture = fixture_fields(retract_keys);
    let retract: FilamentRetractOverrideOptions =
        serde_json::from_value(Value::Object(retract_fixture.clone())).unwrap();
    assert_eq!(serde_json::to_value(retract).unwrap(), Value::Object(retract_fixture));

    let filament_keys = rows
        .iter()
        .filter(|row| row.raw_scope == "filament")
        .map(|row| row.key.as_str());
    let fixture = fixture_fields(filament_keys);
    let filament: FilamentOptions = serde_json::from_value(Value::Object(fixture.clone())).unwrap();
    super::type_assertions::assert_concrete_types(&filament);
    assert_eq!(serde_json::to_vec(&filament).unwrap(), serde_json::to_vec(&fixture).unwrap());
}

#[test]
fn fixture_cardinality_variant_and_semantic_override_sets_are_exact() {
    let rows = inventory();
    let remaining = remaining_rows(&rows);
    let fixture = fixture_fields(remaining.iter().map(|row| row.key.as_str()));
    let lengths = fixture.values().fold(BTreeMap::new(), |mut counts, value| {
        *counts.entry(value.as_array().unwrap().len()).or_insert(0) += 1;
        counts
    });
    assert_eq!(lengths, BTreeMap::from([(2, 42), (8, 27)]));
    assert_eq!(
        fixture
            .iter()
            .filter(|(_, value)| value.as_array().unwrap().len() == 8)
            .map(|(key, _)| key.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(VARIANT_KEYS)
    );

    let overrides = remaining
        .iter()
        .filter(|row| {
            let default = expected_default(row);
            fixture[&row.key]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value != &default[0])
        })
        .map(|row| row.key.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(overrides, BTreeSet::from(FIXTURE_OVERRIDE_KEYS));
    assert_eq!(overrides.len(), 36);
    assert_eq!(remaining.len() - overrides.len(), 33);
}
