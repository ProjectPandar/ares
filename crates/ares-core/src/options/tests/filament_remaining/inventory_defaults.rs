use std::collections::BTreeMap;

use serde_json::{Map, Value};

use super::super::super::{
    FilamentPrintSourceOptions, FilamentRegionSourceOptions, FilamentRetractOverrideOptions,
};
use super::{expected_default, inventory, owner_rows, remaining_rows};

#[test]
fn subgroup_histograms_and_exact_singleton_defaults_match_fixed_tag() {
    let rows = inventory();
    let remaining = remaining_rows(&rows);
    for (owner, expected) in [
        (
            "print_config",
            BTreeMap::from([
                ("coBools", 8),
                ("coEnums", 1),
                ("coFloats", 6),
                ("coInts", 30),
                ("coPercents", 2),
                ("coStrings", 1),
            ]),
        ),
        (
            "print_region_config",
            BTreeMap::from([("coFloats", 3), ("coPercents", 1)]),
        ),
    ] {
        let counts = owner_rows(&remaining, owner).into_iter().fold(
            BTreeMap::new(),
            |mut counts, row| {
                *counts.entry(row.option_type.as_str()).or_insert(0) += 1;
                counts
            },
        );
        assert_eq!(counts, expected, "{owner}");
    }
    let retract = remaining
        .iter()
        .copied()
        .filter(|row| row.static_owner == "unowned" && row.key != "pellet_flow_coefficient")
        .fold(BTreeMap::new(), |mut counts, row| {
            *counts.entry(row.option_type.as_str()).or_insert(0) += 1;
            counts
        });
    assert_eq!(
        retract,
        BTreeMap::from([
            ("coBools", 3),
            ("coEnums", 2),
            ("coFloats", 10),
            ("coPercents", 1),
        ])
    );
    let pellet = remaining
        .iter()
        .filter(|row| row.key == "pellet_flow_coefficient")
        .map(|row| row.option_type.as_str())
        .collect::<Vec<_>>();
    assert_eq!(pellet, ["coFloats"]);

    let mut defaults = Map::new();
    for value in [
        serde_json::to_value(FilamentPrintSourceOptions::default()).unwrap(),
        serde_json::to_value(FilamentRegionSourceOptions::default()).unwrap(),
        serde_json::to_value(FilamentRetractOverrideOptions::default()).unwrap(),
    ] {
        defaults.extend(value.as_object().unwrap().clone());
    }
    defaults.insert(
        "pellet_flow_coefficient".to_owned(),
        serde_json::json!(["0.4157"]),
    );
    for row in remaining {
        assert_eq!(defaults[&row.key], expected_default(row), "{}", row.key);
    }
    assert_eq!(defaults["filament_notes"], Value::Array(vec![Value::String(String::new())]));
}
