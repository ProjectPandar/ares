use std::collections::BTreeSet;

use crate::{ProjectSettings, options::config_export::collector};

use super::super::project_fixture::{project_settings_bytes, project_settings_value};

const EMPTY_NON_NULLABLE_KEYS: [&str; 5] = [
    "bed_exclude_area",
    "head_wrap_detect_zone",
    "parallel_printheads_bed_exclude_areas",
    "post_process",
    "wrapping_exclude_area",
];

fn inventory() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../../../../../tests/ksr_fdmtest_v4/options-v242.json"
    ))
    .unwrap()
}

fn nullable_keys() -> Vec<String> {
    inventory()
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["nullable"] == true)
        .map(|row| row["key"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn config_export_nullable_keeps_all_31_identities_when_empty() {
    let nullable_keys = nullable_keys();
    assert_eq!(nullable_keys.len(), 31);

    let mut raw = project_settings_value();
    let raw = raw.as_object_mut().unwrap();
    for key in &nullable_keys {
        raw.insert(key.clone(), serde_json::json!([]));
    }
    let settings: ProjectSettings = serde_json::from_value(raw.clone().into()).unwrap();
    let entries = collector::collect_config_entries(&settings).unwrap();

    for key in nullable_keys {
        let entry = entries.iter().find(|entry| entry.key == key).unwrap();
        assert_eq!(entry.token, "", "{key}");
        assert!(entry.is_nil, "empty nullable {key} lost its identity");
    }
}

#[test]
fn config_export_nullable_distinguishes_all_nil_mixed_and_non_nullable_empty() {
    let settings: ProjectSettings = serde_json::from_slice(&project_settings_bytes()).unwrap();
    let entries = collector::collect_config_entries(&settings).unwrap();
    let raw = project_settings_value();
    let raw = raw.as_object().unwrap();
    let all_nil_keys = nullable_keys()
        .into_iter()
        .filter(|key| {
            raw[key]
                .as_array()
                .is_some_and(|values| !values.is_empty() && values.iter().all(|value| value == "nil"))
        })
        .collect::<BTreeSet<_>>();

    assert_eq!(all_nil_keys.len(), 15);
    for key in all_nil_keys {
        assert!(
            entries.iter().find(|entry| entry.key == key).unwrap().is_nil,
            "all-nil nullable {key} was retained"
        );
    }
    for key in EMPTY_NON_NULLABLE_KEYS {
        let entry = entries.iter().find(|entry| entry.key == key).unwrap();
        assert_eq!(entry.token, "", "{key}");
        assert!(!entry.is_nil, "empty non-nullable {key} was omitted");
    }

    let mut mixed = project_settings_value();
    mixed.as_object_mut().unwrap().insert(
        "filament_flow_ratio".to_owned(),
        serde_json::json!(["nil", "1"]),
    );
    let mixed: ProjectSettings = serde_json::from_value(mixed).unwrap();
    let mixed = collector::collect_config_entries(&mixed).unwrap();
    let entry = mixed
        .iter()
        .find(|entry| entry.key == "filament_flow_ratio")
        .unwrap();
    assert_eq!(entry.token, "nil,1");
    assert!(!entry.is_nil);
}
