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
    pub compatible_printers_condition: String,
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
    pub printer_model: String,
    pub process: Option<PresetDescriptor>,
    pub embedded: Option<ProjectSettingsRaw>,
    pub direct_stl_machine: Option<ProjectSettingsRaw>,
}

/// Ports the retained-process and direct STL paths of `OrcaSlicer.cpp:2922–2945`.
pub(crate) fn reject_incompatible_load_settings(
    raw: &ProjectSettingsRaw,
    names: &LoadedPresetNames,
    machine_config: &ProjectSettingsRaw,
) -> Result<(), SliceError> {
    let Some(machine) = &names.machine else {
        // No machine file: only branch 4 applies today (process-only
        // lists are rejected by composition upstream-adjacent — branch 3
        // deferred).
        return reject_incompatible_process(raw);
    };
    let new_printer_system_name = machine.system_name();
    // Direct STL has no embedded preset to enter the printer-switch path.
    if let (Some(machine_config), Some(process)) = (&names.direct_stl_machine, &names.process)
        && process.compatible_printers.is_empty()
    {
        return if machine_condition_compatible(
            &process.compatible_printers_condition,
            machine_config,
            new_printer_system_name,
        )? {
            Ok(())
        } else {
            Err(SliceError::ProcessNotCompatible)
        };
    }
    let compatible = if let Some(embedded) = &names.embedded {
        retained_process_compatible(embedded, machine_config, new_printer_system_name)?
    } else if let Some(process) = &names.process {
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

/// `OrcaSlicer.cpp:2908–2945`: translate the embedded process keys, then
/// `Preset.cpp:909–964` checks the supplied machine, not the merged project.
fn retained_process_compatible(
    process: &ProjectSettingsRaw,
    machine: &ProjectSettingsRaw,
    printer_name: &str,
) -> Result<bool, SliceError> {
    let printers = text_list(process, "compatible_printers")
        .filter(|values| !values.is_empty())
        .or_else(|| text_list(process, "print_compatible_printers"))
        .unwrap_or_default();
    if !printers.is_empty() {
        return Ok(printer_name.is_empty() || printers.iter().any(|name| name == printer_name));
    }
    let condition = process
        .get("compatible_printers_condition")
        .and_then(scalar_text)
        .filter(|condition| !condition.is_empty())
        .or_else(|| {
            process
                .get("compatible_machine_expression_group")
                .and_then(|group| group.element(0))
                .and_then(scalar_text)
        })
        .unwrap_or_default();
    machine_condition_compatible(&condition, machine, printer_name)
}

fn machine_condition_compatible(
    condition: &str,
    machine: &ProjectSettingsRaw,
    printer_name: &str,
) -> Result<bool, SliceError> {
    if condition.is_empty() {
        return Ok(true);
    }
    use super::gcode_emit::{
        expression::evaluate_condition,
        value::{Config, Value},
    };
    use crate::OptionValueKind;
    let machine =
        crate::options::json_deserialize::deserialize_preset_config::<ProjectSettingsRaw>(
            machine.clone().into_object(),
        )
        .map_err(SliceError::InvalidInput)?;
    let mut config = Config::default();
    for (key, value) in machine.entries() {
        let Some(definition) = crate::option_definition(key) else {
            continue;
        };
        let value = match definition.kind {
            OptionValueKind::String | OptionValueKind::Enum => {
                scalar_text(value).map(Value::String)
            }
            OptionValueKind::Strings | OptionValueKind::Enums | OptionValueKind::EnumsNullable => {
                list_texts(value)
                    .map(|values| Value::List(values.into_iter().map(Value::String).collect()))
            }
            _ => value.token().and_then(|token| {
                Config::from_block(format!("; {key} = {token}\n").as_bytes())
                    .get(key)
                    .cloned()
            }),
        };
        if let Some(value) = value {
            config.insert(key, value);
        }
    }
    config.insert("printer_preset", Value::String(printer_name.to_owned()));
    if let Some(nozzles) = machine.get("nozzle_diameter") {
        config.insert("num_extruders", Value::Integer(nozzles.len() as i64));
    }
    // Preset.cpp:929–937 accepts a malformed compatibility expression.
    Ok(evaluate_condition(condition, &config, 0).unwrap_or(true))
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
