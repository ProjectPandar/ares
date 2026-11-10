use crate::project::raw_settings::RawConfigValue;

/// One slot of the `different_settings_to_system` vector
/// (`OrcaSlicer.cpp:2838,:2978,:3161`). Slot 0 accumulates the machine-switch
/// `compatible_printers` marker before CLI overrides and mapped tower edits.
pub(super) struct DifferentSettingsContext<'a> {
    pub(super) embedded: Option<&'a RawConfigValue>,
    pub(super) len: usize,
    pub(super) selected: usize,
    pub(super) switch_compatible_printers: bool,
}

impl DifferentSettingsContext<'_> {
    pub(super) fn slot(&self, index: usize) -> RawConfigValue {
        if index == 0 {
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
        if index == self.len - 1 || index <= self.selected {
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
    selected: usize,
) -> RawConfigValue {
    if index >= offset
        && index - offset < selected
        && let Some(value) = group.element(index)
    {
        return value.clone();
    }
    embedded
        .and_then(|value| value.element(index))
        .cloned()
        .unwrap_or_else(|| RawConfigValue::Text(String::new()))
}
