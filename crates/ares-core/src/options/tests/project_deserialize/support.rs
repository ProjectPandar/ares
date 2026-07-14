use std::collections::BTreeSet;

use crate::ProjectSettings;

pub(super) fn serialized_keys(value: &impl serde::Serialize) -> BTreeSet<String> {
    serde_json::to_value(value)
        .unwrap()
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect()
}

pub(super) fn serialized_project_values(
    settings: &ProjectSettings,
) -> serde_json::Map<String, serde_json::Value> {
    let mut values = serde_json::Map::new();
    for group in [
        serde_json::to_value(&settings.printer).unwrap(),
        serde_json::to_value(&settings.process).unwrap(),
        serde_json::to_value(&settings.filament).unwrap(),
        serde_json::to_value(&settings.project).unwrap(),
        serde_json::to_value(&settings.metadata).unwrap(),
    ] {
        for (key, value) in group.as_object().unwrap() {
            assert!(values.insert(key.clone(), value.clone()).is_none());
        }
    }
    values
}
