//! Selected-profile overlay onto a raw settings view
//! (`OrcaSlicer.cpp:2750-2812,3155-3161,3259-3271,3301`).

use crate::{
    ProjectSettings, SliceError,
    project::raw_settings::{ProjectSettingsRaw, RawConfigValue},
};

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
pub(super) fn merged_profile_raw(
    embedded_raw: &ProjectSettingsRaw,
    mut merged: ProjectSettingsRaw,
    profile: &SelectedProfile<'_>,
    selected: usize,
    embedded_count: usize,
) -> Result<ProjectSettingsRaw, SliceError> {
    let SelectedProfile {
        settings,
        profile_raw,
        filament_raws,
        validate_print: _,
    } = profile;
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
    // the covered slots; the tail keeps embedded entries.
    inserts.push((
        "printer_settings_id".to_owned(),
        RawConfigValue::Text(settings.project.preset.printer_settings_id.0.clone()),
    ));
    inserts.push((
        "print_settings_id".to_owned(),
        RawConfigValue::Text(settings.project.preset.print_settings_id.0.clone()),
    ));
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
    // `print_compatible_printers` even when it is absent or empty.
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
    // `:2275,:3680-3699`: mapping every embedded filament to a single
    // preset disables the prime tower (unless a smooth timelapse needs
    // it) and records the key in the process slot.
    let single_preset_mapping = settings
        .project
        .preset
        .filament_settings_id
        .0
        .first()
        .is_some_and(|first| {
            settings
                .project
                .preset
                .filament_settings_id
                .0
                .iter()
                .all(|name| name == first)
        });
    fn truthy(value: Option<&RawConfigValue>) -> bool {
        match value {
            Some(RawConfigValue::Text(value)) => value == "1",
            Some(RawConfigValue::Flag(value)) => *value,
            _ => false,
        }
    }
    let smooth_timelapse = truthy(merged.get("enable_timelapse"))
        && matches!(merged.get("timelapse_type"), Some(RawConfigValue::Text(value)) if value == "1");
    let prime_tower_disabled = single_preset_mapping && !smooth_timelapse;
    if prime_tower_disabled {
        inserts.push((
            "enable_prime_tower".to_owned(),
            RawConfigValue::Text("0".to_owned()),
        ));
    }
    {
        let len = embedded_count + 2;
        let values = (0..len)
            .map(|index| {
                different_settings_slot(
                    embedded_raw.get("different_settings_to_system"),
                    index,
                    len,
                    selected,
                    prime_tower_disabled,
                )
            })
            .collect();
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

/// One slot of a per-slot name vector (`filament_settings_id`, `filament_ids`).
fn name_slot(
    embedded_raw: &ProjectSettingsRaw,
    key: &str,
    loaded: &[String],
    index: usize,
) -> RawConfigValue {
    if index < loaded.len() {
        return RawConfigValue::Text(loaded[index].clone());
    }
    embedded_raw
        .get(key)
        .and_then(|value| value.element(index))
        .cloned()
        .unwrap_or_else(|| RawConfigValue::Text(String::new()))
}

/// One slot of the `inherits_group` vector (`OrcaSlicer.cpp:2817`).
fn inherits_slot(
    inherits: &[String],
    embedded: Option<&RawConfigValue>,
    index: usize,
    len: usize,
    selected: usize,
) -> RawConfigValue {
    let loaded = if index == 0 {
        inherits.first()
    } else if index == len - 1 {
        inherits.last()
    } else if index <= selected {
        inherits.get(index)
    } else {
        None
    };
    loaded
        .map(|text| RawConfigValue::Text(text.clone()))
        .or_else(|| embedded.and_then(|value| value.element(index)).cloned())
        .unwrap_or_else(|| RawConfigValue::Text(String::new()))
}

/// One slot of the `different_settings_to_system` vector
/// (`OrcaSlicer.cpp:2838,:2978,:3161,:3680-3699`).
fn different_settings_slot(
    embedded: Option<&RawConfigValue>,
    index: usize,
    len: usize,
    selected: usize,
    prime_tower_disabled: bool,
) -> RawConfigValue {
    if index == 0 {
        let text = if prime_tower_disabled {
            "enable_prime_tower"
        } else {
            ""
        };
        return RawConfigValue::Text(text.to_owned());
    }
    if index == len - 1 || index <= selected {
        return RawConfigValue::Text(String::new());
    }
    embedded
        .and_then(|value| value.element(index))
        .cloned()
        .unwrap_or_else(|| RawConfigValue::Text(String::new()))
}

/// One slot of a `compatible_*_expression_group` vector
/// (`OrcaSlicer.cpp:3259-3271`).
fn expression_group_slot(
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
