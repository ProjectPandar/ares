//! Selected-profile overlay onto a raw settings view
//! (`OrcaSlicer.cpp:2750-2812,3155-3161,3259-3271,3301`).

mod cli_bookkeeping;
mod device;
mod identity_slots;
mod variant;

use crate::{
    ProjectSettings, SliceError,
    project::raw_settings::{ProjectSettingsRaw, RawConfigValue},
};

use cli_bookkeeping::{DifferentSettingsContext, expression_group_slot};
use identity_slots::{inherits_group, name_slot};
use variant::{SparseVariants, filament_self_index, variant_values};

/// The selected profile's composed pieces shared by the STL and 3MF
/// entries: resolved typed settings, the chain-expanded presence view, and
/// one merged raw per selected filament.
pub(crate) struct SelectedProfile<'a> {
    pub(crate) settings: &'a ProjectSettings,
    pub(crate) profile_raw: &'a ProjectSettingsRaw,
    pub(crate) filament_raws: &'a [ProjectSettingsRaw],
    /// CLI composite entries set this; the 3MF overlay checks the embedded
    /// `post_process` before applying profiles (O5).
    pub(crate) validate_post_process: bool,
    pub(crate) enable_timelapse: bool,
    pub(crate) load_names: Option<&'a crate::project_slice::compatibility::LoadedPresetNames>,
    pub(crate) retained_refresh: Option<crate::profiles::ProjectPresetRefresh<'a>>,
    pub(crate) full_fff_config: Option<&'a crate::profiles::FullFffGroups>,
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
    if !bbl_family_3mf
        || loaded_machine.is_empty()
        || !settings.project.preset.print_settings_id.0.is_empty()
    {
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
pub(super) struct OverlayContext<'a> {
    pub(super) slots: FilamentSlots<'a>,
    pub(super) embedded_count: usize,
    pub(super) machine_switch: Option<MachineSwitch>,
}

#[derive(Clone, Copy)]
pub(super) struct FilamentSlots<'a> {
    pub(super) selected: usize,
    pub(super) filament_indices: &'a [usize],
}

impl FilamentSlots<'_> {
    fn source(self, slot: usize) -> Option<usize> {
        if self.filament_indices.is_empty() {
            (slot < self.selected).then_some(slot)
        } else {
            self.filament_indices.binary_search(&slot).ok()
        }
    }
}

pub(super) fn merged_profile_raw(
    embedded_raw: &ProjectSettingsRaw,
    mut merged: ProjectSettingsRaw,
    profile: &SelectedProfile<'_>,
    context: OverlayContext<'_>,
) -> Result<ProjectSettingsRaw, SliceError> {
    let SelectedProfile {
        settings,
        profile_raw,
        filament_raws,
        ..
    } = profile;
    let OverlayContext {
        slots,
        embedded_count,
        machine_switch,
    } = context;
    let selected = slots.selected;
    let sparse_variants = (!slots.filament_indices.is_empty())
        .then(|| SparseVariants::new(embedded_raw, filament_raws, embedded_count, slots));
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
        if device::is_key(key) {
            continue;
        }
        let values =
            if crate::options::registry::filament_options_with_variant().contains(&key.as_str()) {
                match &sparse_variants {
                    Some(variants) => variants.values(embedded_raw, key),
                    None => variant_values(embedded_raw, filament_raws, key, selected),
                }
            } else {
                let embedded_len = embedded_raw
                    .get(key.as_str())
                    .map_or(0, RawConfigValue::len);
                let loaded_len = if slots.filament_indices.is_empty() {
                    selected
                } else {
                    let source = filament_raws
                        .iter()
                        .rposition(|raw| raw.get(key).is_some())
                        .expect("each filament key comes from at least one loaded file");
                    slots.filament_indices[source] + 1
                };
                let len = embedded_len.max(loaded_len);
                (0..len)
                    .map(|index| {
                        let loaded = slots.source(index).map(|source| &filament_raws[source]);
                        filament_slot(embedded_raw, loaded, key, index)
                    })
                    .collect::<Result<Vec<_>, _>>()?
            };
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
            .map(|index| name_slot(embedded_raw, "print_settings_id", None, index))
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
            .map(|index| {
                let name = slots.source(index).and_then(|source| loaded.get(source));
                name_slot(embedded_raw, key, name, index)
            })
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
    // The selected CLI filament path rebuilds this index after variant replacement
    // (`OrcaSlicer.cpp:3510–3545`, final reconstruction at `3823–3845`).
    if selected > 0 {
        let indices = match &sparse_variants {
            Some(variants) => variants.self_index(),
            None => filament_self_index(embedded_raw, filament_raws, embedded_count, selected),
        };
        inserts.push((
            "filament_self_index".to_owned(),
            RawConfigValue::List(indices),
        ));
    }
    let loaded_kinds = (
        !settings.project.preset.print_settings_id.0.is_empty(),
        !settings.project.preset.printer_settings_id.0.is_empty(),
    );
    if let Some(group) = inherits_group(
        profile_raw,
        embedded_raw,
        embedded_count,
        slots,
        loaded_kinds,
    ) {
        inserts.push(("inherits_group".to_owned(), group));
    }
    let len = embedded_count + 2;
    let context = DifferentSettingsContext {
        embedded: embedded_raw.get("different_settings_to_system"),
        len,
        slots,
        loaded_kinds,
        switch_compatible_printers: machine_switch.is_some(),
    };
    inserts.push((
        "different_settings_to_system".to_owned(),
        RawConfigValue::List((0..len).map(|index| context.slot(index)).collect()),
    ));
    // `:3259-3271`: expression groups are embedded_count-indexed with
    // reserved process/machine slots; loaded filament conditions fill the
    // middle slots.
    for (key, len, offset) in [
        ("compatible_machine_expression_group", embedded_count + 2, 1),
        ("compatible_process_expression_group", embedded_count, 0),
    ] {
        let Some(group) = profile_raw.get(key) else {
            continue;
        };
        let values = (0..len)
            .map(|index| expression_group_slot(group, embedded_raw.get(key), index, offset, slots))
            .collect();
        inserts.push((key.to_owned(), RawConfigValue::List(values)));
    }

    for (key, value) in inserts {
        merged.insert(key, value);
    }

    device::apply(
        &mut merged,
        filament_raws,
        selected > 0 && selected == embedded_count,
    )?;
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

/// One slot of a non-variant filament-vector overlay (`OrcaSlicer.cpp:3293–3301` `set_at`).
fn filament_slot(
    embedded_raw: &ProjectSettingsRaw,
    loaded: Option<&ProjectSettingsRaw>,
    key: &str,
    index: usize,
) -> Result<RawConfigValue, SliceError> {
    if let Some(value) = loaded.and_then(|raw| raw.get(key)) {
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
