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

/// Process | filament slots | machine (`OrcaSlicer.cpp:3203-3204,3228-3231,3377-3380,3603`).
pub(super) fn inherits_group(
    profile: &ProjectSettingsRaw,
    embedded: &ProjectSettingsRaw,
    embedded_count: usize,
    slots: FilamentSlots<'_>,
    (has_process, has_machine): (bool, bool),
) -> Option<RawConfigValue> {
    let loaded = profile.get("inherits_group");
    let embedded = embedded.get("inherits_group");
    if loaded.is_none() && embedded.is_none() {
        return None;
    }
    let loaded_slot = |index: usize| {
        loaded
            .and_then(|group| group.element(index))
            .filter(|value| matches!(value, RawConfigValue::Text(_)))
            .cloned()
            .unwrap_or_else(|| RawConfigValue::Text(String::new()))
    };
    let values = (0..embedded_count + 2)
        .map(|index| {
            let value = match index {
                0 if has_process => Some(loaded_slot(0)),
                index if index == embedded_count + 1 && has_machine => {
                    Some(loaded_slot(slots.selected + 1))
                }
                index if index > 0 && index <= embedded_count => slots
                    .source(index - 1)
                    .map(|source| loaded_slot(source + 1)),
                _ => None,
            };
            value.unwrap_or_else(|| {
                embedded
                    .and_then(|group| group.element(index))
                    .cloned()
                    .unwrap_or_else(|| RawConfigValue::Text(String::new()))
            })
        })
        .collect();
    Some(RawConfigValue::List(values))
}
