use crate::project::raw_settings::RawConfigValue;

use super::FilamentSlots;

/// One slot of the `different_settings_to_system` vector
/// (`OrcaSlicer.cpp:2838,:2978,:3161`). Slot 0 accumulates the machine-switch
/// `compatible_printers` marker before CLI overrides and mapped tower edits.
pub(super) struct DifferentSettingsContext<'a> {
    embedded: Option<&'a RawConfigValue>,
    len: usize,
    slots: FilamentSlots<'a>,
    loaded_settings: (bool, bool),
    switch_compatible_printers: bool,
}

impl<'a> DifferentSettingsContext<'a> {
    pub(super) fn new(
        embedded: Option<&'a RawConfigValue>,
        len: usize,
        slots: FilamentSlots<'a>,
        preset: &crate::options::ProjectPresetSourceOptions,
        switch_compatible_printers: bool,
    ) -> Self {
        Self {
            embedded,
            len,
            slots,
            loaded_settings: (
                !preset.print_settings_id.0.is_empty(),
                !preset.printer_settings_id.0.is_empty(),
            ),
            switch_compatible_printers,
        }
    }

    pub(super) fn slot(&self, index: usize) -> RawConfigValue {
        let (process_loaded, machine_loaded) = self.loaded_settings;
        if index == 0 {
            if process_loaded {
                return RawConfigValue::Text(String::new());
            }
            let existing = self
                .embedded
                .and_then(|value| value.element(0))
                .and_then(|value| match value {
                    RawConfigValue::Text(text) => Some(text.as_str()),
                    _ => None,
                })
                .unwrap_or("");
            let mut keys: Vec<String> = existing
                .split(';')
                .filter(|key| !key.is_empty())
                .map(str::to_owned)
                .collect();
            if self.switch_compatible_printers
                && !keys.iter().any(|key| key == "compatible_printers")
            {
                keys.insert(0, "compatible_printers".to_owned());
            }
            return RawConfigValue::Text(keys.join(";"));
        }
        if (index == self.len - 1 && machine_loaded)
            || (index < self.len - 1 && self.slots.source(index - 1).is_some())
        {
            return RawConfigValue::Text(String::new());
        }
        self.embedded
            .and_then(|value| value.element(index))
            .cloned()
            .unwrap_or_else(|| RawConfigValue::Text(String::new()))
    }
}

/// One slot of a `compatible_*_expression_group` vector
/// (`OrcaSlicer.cpp:3259-3271`).
pub(super) fn expression_group_slot(
    group: &RawConfigValue,
    embedded: Option<&RawConfigValue>,
    index: usize,
    offset: usize,
    slots: FilamentSlots<'_>,
) -> RawConfigValue {
    if let Some(source) = index
        .checked_sub(offset)
        .and_then(|slot| slots.source(slot))
        && let Some(value) = group.element(source + offset)
    {
        return value.clone();
    }
    embedded
        .and_then(|value| value.element(index))
        .cloned()
        .unwrap_or_else(|| RawConfigValue::Text(String::new()))
}
