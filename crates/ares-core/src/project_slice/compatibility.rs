//! Plain-3MF `print_compatible_printers` load gate
//! (`OrcaSlicer.cpp:2562-2620` fourth branch; `-17` exit at `:2653-2654`).

use crate::{
    SliceError,
    project::raw_settings::{ProjectSettingsRaw, RawConfigValue},
};

/// Fourth-branch adjudication of a plain CLI 3MF slice: compatible unless
/// the embedded `print_compatible_printers` (`OrcaSlicer.cpp:1770`) is a
/// non-empty list that does not contain the current printer's system name
/// (`:2608-2620`). An absent or empty list keeps the old-3mf compatible
/// rule (`:2614-2618`); rejection is `CLI_PROCESS_NOT_COMPATIBLE`
/// (`:2653-2654`, `Utils.hpp:42`).
pub(super) fn reject_incompatible_process(raw: &ProjectSettingsRaw) -> Result<(), SliceError> {
    let Some(compatible) = text_list(raw, "print_compatible_printers") else {
        return Ok(());
    };
    if compatible.is_empty() {
        return Ok(());
    }
    let printer = printer_system_name(raw);
    if compatible.contains(&printer) {
        Ok(())
    } else {
        Err(SliceError::ProcessNotCompatible)
    }
}

/// The printer's system name: the printer slot of `inherits_group` — the
/// last element of `[process, filaments…, printer]` (`OrcaSlicer.cpp:1741-1753`)
/// — falling back to `printer_settings_id` when the slot or the whole
/// group is absent or empty.
fn printer_system_name(raw: &ProjectSettingsRaw) -> String {
    raw.get("inherits_group")
        .and_then(list_texts)
        .and_then(|group| group.into_iter().last())
        .filter(|name| !name.is_empty())
        .or_else(|| raw.get("printer_settings_id").and_then(scalar_text))
        .unwrap_or_default()
}

fn text_list(raw: &ProjectSettingsRaw, key: &str) -> Option<Vec<String>> {
    raw.get(key).and_then(list_texts)
}

/// Normalized `Strings` values are `List`s of text slots; a scalar text
/// kept by the raw view reads as its single element.
fn list_texts(value: &RawConfigValue) -> Option<Vec<String>> {
    match value {
        RawConfigValue::List(values) => Some(
            values
                .iter()
                .map(|item| match item {
                    RawConfigValue::Text(text) => text.clone(),
                    _ => String::new(),
                })
                .collect(),
        ),
        RawConfigValue::Text(text) => Some(vec![text.clone()]),
        _ => None,
    }
}

fn scalar_text(value: &RawConfigValue) -> Option<String> {
    match value {
        RawConfigValue::Text(text) => Some(text.clone()),
        RawConfigValue::List(values) => match values.as_slice() {
            [RawConfigValue::Text(text)] => Some(text.clone()),
            _ => None,
        },
        _ => None,
    }
}
