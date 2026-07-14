use std::{collections::BTreeSet, fmt::Write};

use crate::ProjectSettings;

use super::super::project_fixture::project_settings_bytes;
use super::support::{serialized_keys, serialized_project_values};

#[test]
fn project_settings_accepts_all_real_canonical_members_with_exact_ownership() {
    let raw = project_settings_bytes();
    let fixture: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    let fixture = fixture.as_object().unwrap();
    let settings: ProjectSettings = serde_json::from_slice(&raw).unwrap();

    let printer = serialized_keys(&settings.printer);
    let process = serialized_keys(&settings.process);
    let filament = serialized_keys(&settings.filament);
    let project = serialized_keys(&settings.project);
    let metadata = serialized_keys(&settings.metadata);

    assert_eq!(fixture.len(), 653);
    assert_eq!(printer.len(), 132);
    assert_eq!(process.len(), 352);
    assert_eq!(filament.len(), 122);
    assert_eq!(project.len(), 44);
    assert_eq!(metadata.len(), 3);

    let mut owned = BTreeSet::new();
    for group in [&printer, &process, &filament, &project, &metadata] {
        for key in group {
            assert!(owned.insert(key.clone()), "duplicate owner for {key}");
        }
    }
    assert_eq!(
        owned,
        fixture.keys().cloned().collect::<BTreeSet<String>>()
    );
}

#[test]
fn project_settings_accepts_arbitrary_member_order() {
    let raw = project_settings_bytes();
    let fixture: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    let fixture = fixture.as_object().unwrap();
    let mut reversed = String::from("{");
    for (index, (key, value)) in fixture.iter().rev().enumerate() {
        if index != 0 {
            reversed.push(',');
        }
        write!(
            reversed,
            "{}:{}",
            serde_json::to_string(key).unwrap(),
            serde_json::to_string(value).unwrap()
        )
        .unwrap();
    }
    reversed.push('}');

    let expected: ProjectSettings = serde_json::from_slice(&raw).unwrap();
    let actual: ProjectSettings = serde_json::from_str(&reversed).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn omitted_project_settings_members_use_group_defaults() {
    let settings: ProjectSettings = serde_json::from_str("{}").unwrap();
    assert_eq!(settings, ProjectSettings::default());
}

#[test]
fn project_settings_accepts_every_inventory_wire_family() {
    let raw = project_settings_bytes();
    let fixture: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    let fixture = fixture.as_object().unwrap();
    let inventory: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../../tests/ksr_fdmtest_v4/options-v242.json"
    ))
    .unwrap();
    let rows = inventory.as_array().unwrap();
    let families = rows
        .iter()
        .map(|row| row["option_type"].as_str().unwrap())
        .collect::<BTreeSet<_>>();
    let defaults = serialized_project_values(&ProjectSettings::default());
    let mut representatives = String::from("{");
    let mut selected = Vec::new();

    for family in &families {
        let row = rows
            .iter()
            .find(|row| {
                row["option_type"].as_str() == Some(family)
                    && fixture[row["key"].as_str().unwrap()]
                        != defaults[row["key"].as_str().unwrap()]
            })
            .unwrap_or_else(|| panic!("missing nondefault fixture representative for {family}"));
        if representatives.len() > 1 {
            representatives.push(',');
        }
        let key = row["key"].as_str().unwrap();
        write!(
            representatives,
            "{}:{}",
            serde_json::to_string(key).unwrap(),
            serde_json::to_string(&fixture[key]).unwrap()
        )
        .unwrap();
        selected.push((family, key));
    }
    representatives.push('}');

    assert_eq!(families.len(), 17);
    let parsed: ProjectSettings = serde_json::from_str(&representatives).unwrap();
    let parsed_values = serialized_project_values(&parsed);
    for (family, key) in selected {
        assert_eq!(
            parsed_values[key], fixture[key],
            "{family} representative {key} was consumed without preserving its value"
        );
    }
}
