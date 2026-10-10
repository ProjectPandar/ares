//! `print_compatible_printers` load gates
//! (`OrcaSlicer.cpp:2575-2620`; `-17` exit at `:2650-2654`).

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

/// Loaded `--load-settings` preset descriptor: only the fields the
/// adjudication reads (`OrcaSlicer.cpp:2034-2098`).
#[derive(Clone)]
pub(crate) struct PresetDescriptor {
    /// `config_name` (`load_config_file` result).
    pub name: String,
    /// `from`: `system` names itself the system preset
    /// (`:2041-2044,:2081-2084`).
    pub from: String,
    /// `inherits`: the system name for user/User-scoped files
    /// (`:2045,:2085`).
    pub inherits: Option<String>,
    /// `compatible_printers` (`:2090`), process files only.
    pub compatible_printers: Vec<String>,
}

impl PresetDescriptor {
    /// `new_*_system_name` (`OrcaSlicer.cpp:2041-2046,:2081-2086`).
    fn system_name(&self) -> &str {
        if self.from == "system" {
            &self.name
        } else {
            self.inherits.as_deref().unwrap_or_default()
        }
    }
}

/// The `new_*` names the `--load-settings` file loop produced
/// (`OrcaSlicer.cpp:2023-2106`): `None` when the list carried no file of
/// that kind.
#[derive(Clone)]
pub(crate) struct LoadedPresetNames {
    pub machine: Option<PresetDescriptor>,
    pub process: Option<PresetDescriptor>,
}

/// Loaded-names adjudication (`OrcaSlicer.cpp:2575-2620` branches 1–2;
/// `-17` at `:2650-2654`). Branches 3–4 and the `-16` upward gate are
/// handled elsewhere/deferred: branch 3 (process file only) currently
/// cannot arrive because the composition accepts only complete lists —
/// treat it like branch 2's "same-name machine" path is impossible, and
/// branch 4 is `reject_incompatible_process` for the plain path.
///
/// With a loaded machine file (`!new_printer_name.empty()`):
/// - branch 1 (`:2576-2584`): `new_process_name` non-empty →
///   `new_print_compatible_printers ∋ new_printer_system_name`;
/// - branch 2 (`:2586-2594`): process empty →
///   `current_print_compatible_printers ∋ new_printer_system_name`;
/// - the `machine_switch` relaxation (`:2624-2627`) then accepts any
///   `new_printer_name != current_printer_name` — the upward `-16`
///   check (`:2630-2643`) is deferred.
pub(super) fn reject_incompatible_load_settings(
    raw: &ProjectSettingsRaw,
    names: &LoadedPresetNames,
) -> Result<(), SliceError> {
    let Some(machine) = &names.machine else {
        // No machine file: only branch 4 applies today (process-only
        // lists are rejected by composition upstream-adjacent — branch 3
        // deferred).
        return reject_incompatible_process(raw);
    };
    let new_printer_system_name = machine.system_name();
    let compatible = if let Some(process) = &names.process {
        process
            .compatible_printers
            .iter()
            .any(|entry| entry == new_printer_system_name)
    } else {
        text_list(raw, "print_compatible_printers")
            .is_some_and(|list| list.iter().any(|entry| entry == new_printer_system_name))
    };
    if compatible {
        return Ok(());
    }
    // `OrcaSlicer.cpp:2624-2627`: a different named machine file switches
    // the printer; the upward `-16` sub-path stays deferred.
    let current_printer_name = raw
        .get("printer_settings_id")
        .and_then(scalar_text)
        .unwrap_or_default();
    if !machine.name.is_empty()
        && !current_printer_name.is_empty()
        && machine.name != current_printer_name
    {
        return Ok(());
    }
    Err(SliceError::ProcessNotCompatible)
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
