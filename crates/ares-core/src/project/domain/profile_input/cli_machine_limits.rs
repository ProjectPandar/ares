//! OrcaSlicer.cpp:3254–3331, explicit printer `update_full_config` cap.

use std::{collections::BTreeMap, sync::LazyLock};

use serde::Deserialize;

use crate::{
    OptionValueKind, option_definition,
    options::{OrcaFloat, OrcaFloats, json_deserialize::decode},
    project::raw_settings::{ProjectSettingsRaw, RawConfigValue},
    project_slice::compatibility::LoadedPresetNames,
};

#[derive(Deserialize)]
struct CliConfig {
    printer: BTreeMap<String, Printer>,
}

#[derive(Deserialize)]
struct Printer {
    #[serde(default)]
    machine_limits: BTreeMap<String, String>,
}

static CONFIG: LazyLock<Option<CliConfig>> =
    LazyLock::new(|| serde_json::from_str(include_str!("data/BBL/cli_config.json")).ok());

pub(super) fn apply(
    raw: &mut ProjectSettingsRaw,
    loaded: Option<&LoadedPresetNames>,
    original_name: &str,
) -> bool {
    let Some(loaded) = loaded else {
        return false;
    };
    if loaded.printer_model.is_empty()
        || loaded
            .machine
            .as_ref()
            .is_none_or(|machine| machine.name.is_empty() || machine.name == original_name)
    {
        return false;
    }
    let Some(printer) = CONFIG
        .as_ref()
        .and_then(|config| config.printer.get(&loaded.printer_model))
    else {
        return false;
    };
    let mut changed = false;
    for (source, cap) in &printer.machine_limits {
        let Some(suffix) = source.strip_prefix("cli_safe") else {
            continue;
        };
        let key = format!("machine_max{suffix}");
        if !option_definition(&key).is_some_and(|def| def.kind == OptionValueKind::Floats) {
            continue;
        }
        if let Some(value) = raw.get(&key).and_then(|value| cap_slots(value, cap)) {
            raw.insert(key, value);
            changed = true;
        }
    }
    changed
}

fn cap_slots(value: &RawConfigValue, cap: &str) -> Option<RawConfigValue> {
    let OrcaFloats(caps) =
        OrcaFloats::deserialize(decode(OptionValueKind::Floats, cap).ok()?).ok()?;
    let mut slots = match value {
        RawConfigValue::Text(wire) => match decode(OptionValueKind::Floats, wire).ok()? {
            RawConfigValue::List(values) => values,
            _ => return None,
        },
        RawConfigValue::List(values) => values.clone(),
        _ => return None,
    };
    let mut changed = false;
    for (slot, OrcaFloat(cap)) in slots.iter_mut().zip(caps) {
        if cap != 0.0 && OrcaFloat::deserialize(slot.clone()).is_ok_and(|current| cap < current.0) {
            *slot = RawConfigValue::Text(cap.to_string());
            changed = true;
        }
    }
    changed.then_some(RawConfigValue::List(slots))
}
