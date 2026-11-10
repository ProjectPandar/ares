use crate::project::raw_settings::{ProjectSettingsRaw, RawConfigValue};

/// One slot of a per-slot name vector (`filament_settings_id`, `filament_ids`).
pub(super) fn name_slot(
    embedded_raw: &ProjectSettingsRaw,
    key: &str,
    loaded: &[String],
    index: usize,
) -> RawConfigValue {
    if index < loaded.len() {
        return RawConfigValue::Text(loaded[index].clone());
    }
    embedded_raw
        .get(key)
        .and_then(|value| value.element(index))
        .cloned()
        .unwrap_or_else(|| RawConfigValue::Text(String::new()))
}

/// One slot of the `inherits_group` vector (`OrcaSlicer.cpp:2817`).
pub(super) fn inherits_slot(
    inherits: &[String],
    embedded: Option<&RawConfigValue>,
    index: usize,
    len: usize,
    selected: usize,
) -> RawConfigValue {
    let loaded = if index == 0 {
        inherits.first()
    } else if index == len - 1 {
        inherits.last()
    } else if index <= selected {
        inherits.get(index)
    } else {
        None
    };
    loaded
        .map(|text| RawConfigValue::Text(text.clone()))
        .or_else(|| embedded.and_then(|value| value.element(index)).cloned())
        .unwrap_or_else(|| RawConfigValue::Text(String::new()))
}
