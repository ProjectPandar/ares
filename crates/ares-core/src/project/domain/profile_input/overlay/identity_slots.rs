use crate::project::raw_settings::{ProjectSettingsRaw, RawConfigValue};

use super::FilamentSlots;

/// One slot of a per-slot name vector (`filament_settings_id`, `filament_ids`).
pub(super) fn name_slot(
    embedded_raw: &ProjectSettingsRaw,
    key: &str,
    loaded: Option<&String>,
    index: usize,
) -> RawConfigValue {
    if let Some(loaded) = loaded {
        return RawConfigValue::Text(loaded.clone());
    }
    embedded_raw
        .get(key)
        .and_then(|value| value.element(index))
        .cloned()
        .unwrap_or_else(|| RawConfigValue::Text(String::new()))
}

/// Process | filament slots | machine (`OrcaSlicer.cpp:3592-3603`).
pub(super) fn inherits_group(
    profile: &ProjectSettingsRaw,
    embedded: &ProjectSettingsRaw,
    embedded_count: usize,
    slots: FilamentSlots<'_>,
) -> Option<RawConfigValue> {
    let group = profile.get("inherits_group")?;
    let loaded = (0..slots.selected + 2)
        .filter_map(|index| match group.element(index) {
            Some(value @ RawConfigValue::Text(_)) => Some(value),
            _ => None,
        })
        .collect::<Vec<_>>();
    if loaded.is_empty() {
        return None;
    }
    let embedded = embedded.get("inherits_group");
    let values = (0..embedded_count + 2)
        .map(|index| {
            let value = match index {
                0 if slots.selected > 0 => loaded.first().copied(),
                0 => None,
                index if index == embedded_count + 1 => loaded.last().copied(),
                index => slots
                    .source(index - 1)
                    .and_then(|source| loaded.get(source + 1).copied()),
            };
            value
                .or_else(|| embedded.and_then(|group| group.element(index)))
                .cloned()
                .unwrap_or_else(|| RawConfigValue::Text(String::new()))
        })
        .collect();
    Some(RawConfigValue::List(values))
}
