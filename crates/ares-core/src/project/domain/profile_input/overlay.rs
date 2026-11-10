//! Selected-profile overlay onto a raw settings view
//! (`OrcaSlicer.cpp:2750-2812,3155-3161,3259-3271,3301`).

mod cli_bookkeeping;
mod identity_slots;

use crate::{
    ProjectSettings, SliceError,
    project::raw_settings::{ProjectSettingsRaw, RawConfigValue},
};

use cli_bookkeeping::{DifferentSettingsContext, expression_group_slot};
use identity_slots::{inherits_slot, name_slot};

/// The selected profile's composed pieces shared by the STL and 3MF
/// entries: resolved typed settings, the chain-expanded presence view, and
/// one merged raw per selected filament.
pub(crate) struct SelectedProfile<'a> {
    pub(crate) settings: &'a ProjectSettings,
    pub(crate) profile_raw: &'a ProjectSettingsRaw,
    pub(crate) filament_raws: &'a [ProjectSettingsRaw],
    /// CLI composite entries set this; the 3MF overlay checks the embedded
    /// `post_process` before applying profiles (O5).
    pub(crate) validate_print: bool,
}

/// The selected-profile overlay onto a raw settings view:
/// non-filament keys overwrite, filament keys fill per loaded slot, and
/// the identity/dynamic-group keys are rebuilt (`OrcaSlicer.cpp:2750-2812,
/// 3155-3161,3259-3271,3301`). `merged` seeds from the embedded view and
/// is returned with every profile-supplied key applied.
/// The `machine_switch` state the merge consumes
/// (`OrcaSlicer.cpp:2622-2640`): an incompatible loaded machine with no
/// loaded process forces compatibility, and the config-application stage
/// records the switch. `rename_process` carries the `is_bbl_3mf` gate of
/// the `"(auto)"` preset rename (`:2660-2702`).
pub(super) struct MachineSwitch {
    pub(super) system_name: String,
    pub(super) rename_process: bool,
}

/// `OrcaSlicer.cpp:2622-2640,2585-2596`: the merge-side machine-switch —
/// a loaded machine (typed `printer_settings_id` non-empty) with no
/// loaded process, differing from the embedded printer, and not already
/// listed in the embedded `print_compatible_printers`. The `-17`
/// rejection adjudication itself stays in `project_slice/compatibility.rs`.
pub(super) fn machine_switch(
    embedded_raw: &ProjectSettingsRaw,
    settings: &ProjectSettings,
    bbl_family_3mf: bool,
) -> Option<MachineSwitch> {
    let loaded_machine = settings.project.preset.printer_settings_id.0.as_str();
    if loaded_machine.is_empty() || !settings.project.preset.print_settings_id.0.is_empty() {
        return None;
    }
    let embedded_printer = embedded_raw
        .get("printer_settings_id")
        .and_then(|value| value.element(0))
        .and_then(|value| match value {
            crate::project::raw_settings::RawConfigValue::Text(text) => Some(text),
            _ => None,
        })?;
    if embedded_printer.is_empty() || embedded_printer == loaded_machine {
        return None;
    }
    let embedded_compatible = embedded_raw
        .get("print_compatible_printers")
        .is_some_and(|value| {
            (0..value.len()).any(|index| {
                matches!(
                    value.element(index),
                    Some(crate::project::raw_settings::RawConfigValue::Text(name))
                        if name == loaded_machine
                )
            })
        });
    if embedded_compatible {
        return None;
    }
    Some(MachineSwitch {
        system_name: loaded_machine.to_owned(),
        rename_process: bbl_family_3mf,
    })
}

/// The counts and machine-switch state the overlay needs beyond the
/// selected profile itself.
pub(super) struct OverlayContext {
    pub(super) selected: usize,
    pub(super) embedded_count: usize,
    pub(super) machine_switch: Option<MachineSwitch>,
}

pub(super) fn merged_profile_raw(
    embedded_raw: &ProjectSettingsRaw,
    mut merged: ProjectSettingsRaw,
    profile: &SelectedProfile<'_>,
    context: OverlayContext,
) -> Result<ProjectSettingsRaw, SliceError> {
    let SelectedProfile {
        settings,
        profile_raw,
        filament_raws,
        validate_print: _,
    } = profile;
    let OverlayContext {
        selected,
        embedded_count,
        machine_switch,
    } = context;
    let filament_keys: std::collections::BTreeSet<&String> = filament_raws
        .iter()
        .flat_map(|raw| raw.entries().map(|(key, _)| key))
        .collect();
    // Dynamic groups whose slot layout follows the embedded filament
    // count; they are rebuilt below.
    let dynamic_groups = [
        "different_settings_to_system",
        "inherits_group",
        "compatible_machine_expression_group",
        "compatible_process_expression_group",
    ];
    for (key, value) in profile_raw.entries() {
        if filament_keys.contains(key) || dynamic_groups.contains(&key.as_str()) {
            continue;
        }
        merged.insert(key.clone(), value.clone());
    }

    // Filament keys overlay per loaded slot (`set_at(src, index - 1, 0)`):
    // supplied slots take the filament's first element, the rest keep the
    // embedded element (front-copy for growth, `Config.hpp:427-447`). A key
    // absent from the embedded config leaves `Null` placeholders in slots
    // no filament supplies; Orca-exported projects embed the full config,
    // so that case is bounded to synthetic inputs.
    let mut inserts: Vec<(String, RawConfigValue)> = Vec::new();
    for key in &filament_keys {
        let embedded_len = embedded_raw
            .get(key.as_str())
            .map_or(0, RawConfigValue::len);
        let len = embedded_len.max(selected);
        let values = (0..len)
            .map(|index| filament_slot(embedded_raw, filament_raws, key, index, selected))
            .collect::<Result<Vec<_>, _>>()?;
        inserts.push(((*key).clone(), RawConfigValue::List(values)));
    }

    // Settings IDs (`:2048,:2088,:3155-3158`) carry the selected names into
    // the covered slots; the tail keeps embedded entries. A not-loaded
    // kind keeps its empty name (`:2009-2106` arms run per loaded file),
    // so the embedded identity entry is retained untouched.
    if !settings.project.preset.printer_settings_id.0.is_empty() {
        inserts.push((
            "printer_settings_id".to_owned(),
            RawConfigValue::Text(settings.project.preset.printer_settings_id.0.clone()),
        ));
    }
    if !settings.project.preset.print_settings_id.0.is_empty() {
        inserts.push((
            "print_settings_id".to_owned(),
            RawConfigValue::Text(settings.project.preset.print_settings_id.0.clone()),
        ));
    } else if machine_switch
        .as_ref()
        .is_some_and(|switch| switch.rename_process)
    {
        // `OrcaSlicer.cpp:2660-2702`: the bbl machine-switch renames the
        // retained embedded process preset with the `"(auto)"` suffix and
        // writes it into the config (`:2700-2702`).
        let renamed = (0..embedded_count)
            .map(|index| name_slot(embedded_raw, "print_settings_id", &[], index))
            .map(|slot| match slot {
                RawConfigValue::Text(name) if !name.is_empty() => {
                    RawConfigValue::Text(format!("{name}(auto)"))
                }
                other => other,
            })
            .collect::<Vec<_>>();
        if !renamed.is_empty() {
            inserts.push((
                "print_settings_id".to_owned(),
                RawConfigValue::List(renamed),
            ));
        }
    }
    for (key, loaded) in [
        (
            "filament_settings_id",
            &settings.project.preset.filament_settings_id.0,
        ),
        ("filament_ids", &settings.project.gcode.filament_ids.0),
    ] {
        let embedded_len = embedded_raw.get(key).map_or(0, RawConfigValue::len);
        let len = embedded_len.max(embedded_count);
        let values = (0..len)
            .map(|index| name_slot(embedded_raw, key, loaded, index))
            .collect();
        inserts.push((key.to_owned(), RawConfigValue::List(values)));
    }
    // `:2980-2984`: the loaded process's `compatible_printers` overwrites
    // `print_compatible_printers` even when it is absent or empty; with no
    // process loaded the embedded list is retained untouched — except the
    // machine-switch append (`:3051`), which pushes the loaded machine's
    // system name onto the embedded list.
    if !settings.project.preset.print_settings_id.0.is_empty() {
        inserts.push((
            "print_compatible_printers".to_owned(),
            RawConfigValue::List(
                settings
                    .project
                    .preset
                    .print_compatible_printers
                    .0
                    .iter()
                    .map(|value| RawConfigValue::Text(value.clone()))
                    .collect(),
            ),
        ));
    } else if let Some(system_name) = machine_switch
        .as_ref()
        .map(|switch| switch.system_name.clone())
    {
        let len = embedded_raw
            .get("print_compatible_printers")
            .map_or(0, RawConfigValue::len);
        let mut values = (0..len)
            .map(|index| {
                embedded_raw
                    .get("print_compatible_printers")
                    .and_then(|value| value.element(index))
                    .cloned()
                    .unwrap_or_else(|| RawConfigValue::Text(String::new()))
            })
            .collect::<Vec<_>>();
        values.push(RawConfigValue::Text(system_name));
        inserts.push((
            "print_compatible_printers".to_owned(),
            RawConfigValue::List(values),
        ));
    }
    // `:3117-3337`: single-variant selections rebuild `filament_self_index`
    // as a plain 1..=filament_count identity.
    inserts.push((
        "filament_self_index".to_owned(),
        RawConfigValue::List(
            (1..=embedded_count)
                .map(|index| RawConfigValue::Text(index.to_string()))
                .collect(),
        ),
    ));

    // `:2817-2818`: group vectors are embedded_count + 2, indexed
    // process | filaments | machine; only the loaded slots are written.
    let inherits: Vec<String> = (0..selected + 2)
        .filter_map(|index| {
            match profile_raw
                .get("inherits_group")
                .and_then(|value| value.element(index))
            {
                Some(RawConfigValue::Text(text)) => Some(text.clone()),
                _ => None,
            }
        })
        .collect();
    if !inherits.is_empty() {
        let len = embedded_count + 2;
        let values = (0..len)
            .map(|index| {
                inherits_slot(
                    &inherits,
                    embedded_raw.get("inherits_group"),
                    index,
                    len,
                    selected,
                )
            })
            .collect();
        inserts.push(("inherits_group".to_owned(), RawConfigValue::List(values)));
    }
    // `:2838,:2978,:3161`: the process slot, loaded filament slots, and the
    // machine slot are cleared; unloaded filament slots stay embedded.
    {
        let len = embedded_count + 2;
        let context = DifferentSettingsContext {
            embedded: embedded_raw.get("different_settings_to_system"),
            len,
            selected,
            switch_compatible_printers: machine_switch.is_some(),
        };
        let values = (0..len).map(|index| context.slot(index)).collect();
        inserts.push((
            "different_settings_to_system".to_owned(),
            RawConfigValue::List(values),
        ));
    }
    // `:3259-3271`: expression groups are embedded_count-indexed with
    // reserved process/machine slots; loaded filament conditions fill the
    // middle slots.
    // `cli_metadata::filament_conditions` lays the loaded conditions at
    // [process | filaments | machine] for the machine group and
    // [filaments] for the process group.
    for (key, len, offset) in [
        (
            "compatible_machine_expression_group",
            embedded_count + 2,
            1usize,
        ),
        (
            "compatible_process_expression_group",
            embedded_count,
            0usize,
        ),
    ] {
        let Some(group) = profile_raw.get(key) else {
            continue;
        };
        let values = (0..len)
            .map(|index| {
                expression_group_slot(group, embedded_raw.get(key), index, offset, selected)
            })
            .collect();
        inserts.push((key.to_owned(), RawConfigValue::List(values)));
    }

    for (key, value) in inserts {
        merged.insert(key, value);
    }

    Ok(merged)
}

/// Keys upstream skips at the per-filament merge (`OrcaSlicer.cpp:3293`);
/// their emptiness is legal — the compatibility gate treats an empty
/// `compatible_printers` as unrestricted, so they never reach `set_at`.
const SKIPPED_BY_SET_AT: [&str; 5] = [
    "compatible_printers",
    "compatible_prints",
    "model_id",
    "dev_model_name",
    "filament_settings_id",
];

/// One slot of a filament-vector overlay (`OrcaSlicer.cpp:3301` `set_at`).
fn filament_slot(
    embedded_raw: &ProjectSettingsRaw,
    filament_raws: &[ProjectSettingsRaw],
    key: &str,
    index: usize,
    selected: usize,
) -> Result<RawConfigValue, SliceError> {
    if index < selected
        && let Some(value) = filament_raws[index].get(key)
    {
        // `set_at(src, index - 1, 0)` throws when the source vector is
        // empty (`Config.hpp:432-441`); upstream aborts on the uncaught
        // exception while Ares returns the same message as `InvalidInput`
        // (accepted deviation, see the O8 ARD).
        if matches!(value, RawConfigValue::List(values) if values.is_empty())
            && !SKIPPED_BY_SET_AT.contains(&key)
        {
            return Err(SliceError::InvalidInput(
                "ConfigOptionVector::set_at(): Assigning from an empty vector".to_owned(),
            ));
        }
        if let Some(element) = value.element(0) {
            return Ok(element.clone());
        }
    }
    Ok(embedded_raw
        .get(key)
        .and_then(|value| value.element(index).or_else(|| value.element(0)))
        .cloned()
        .unwrap_or(RawConfigValue::Null))
}
