use std::collections::BTreeSet;

use super::super::{ProfileKind, catalog::Catalog, preset_normalization};
use super::{PresetLookup, filament, keys};
use crate::{
    SliceError,
    options::overrides::dirty_columns,
    project::raw_settings::{ProjectSettingsRaw, RawConfigValue},
};

// `OrcaSlicer.cpp:668–669`: filament members require their own merge branch.
const GCODES_KEY_SET: [&str; 12] = [
    "machine_start_gcode",
    "machine_end_gcode",
    "machine_pause_gcode",
    "change_filament_gcode",
    "before_layer_change_gcode",
    "layer_change_gcode",
    "template_custom_gcode",
    "printing_by_object_gcode",
    "time_lapse_gcode",
    "wrapping_detection_gcode",
    "filament_start_gcode",
    "filament_end_gcode",
];
const MACHINE_GCODES: &[&str] = GCODES_KEY_SET.split_at(10).0;

pub(super) fn is_gcode_key(key: &str) -> bool {
    GCODES_KEY_SET.contains(&key)
}

pub(super) enum RefreshScope<'a> {
    Project(Option<&'a crate::ProjectSettings>),
    Filaments,
}

pub(super) fn refresh(
    lookup: PresetLookup<'_>,
    project: &ProjectSettingsRaw,
    uptodate: bool,
    scope: RefreshScope<'_>,
) -> Result<(ProjectSettingsRaw, bool), SliceError> {
    let names = strings(project, "filament_settings_id");
    let inherited = strings(project, "inherits_group");
    let dirty = strings(project, "different_settings_to_system");
    let mut dirty_out = dirty.clone();
    let mut dirty_changed = false;
    let refresh_unlisted = uptodate || dirty.iter().any(|entry| !entry.is_empty());
    let mut result = project.clone();
    let mut retained_thumbnails = false;
    // CLI:3167–3200: machine and process refresh, not replacement selection.
    let supplied = match scope {
        RefreshScope::Project(selected) => selected.map(|selected| &selected.project.preset),
        RefreshScope::Filaments => None,
    };
    for (kind, identity_key, slot, replaced) in [
        (
            ProfileKind::Machine,
            "printer_settings_id",
            names.len() + 1,
            supplied.is_some_and(|p| !p.printer_settings_id.0.is_empty()),
        ),
        (
            ProfileKind::Process,
            "print_settings_id",
            0,
            supplied.is_some_and(|p| !p.print_settings_id.0.is_empty()),
        ),
    ] {
        if replaced || matches!(scope, RefreshScope::Filaments) {
            continue;
        }
        let identity = strings(project, identity_key);
        let name = identity.first().map(String::as_str).unwrap_or("");
        let name = system_name(name, &inherited, slot);
        if name.is_empty() {
            continue;
        }
        let native_file_name = uptodate
            .then(|| lookup.native_file_name(kind, slot))
            .flatten();
        // CLI:3182-3185: a resolver miss keeps the project's values.
        let Some(mut system) = lookup.system(kind, name, slot)? else {
            continue;
        };
        let skip_gcodes = native_file_name.is_some() && lookup.skip_modified_gcodes();
        // CLI:3216-3222,3348-3365 decode only authored overrides for flat
        // refresh files; `load_project_preset` (`:3179`) adds identity keys.
        let mut protected = if native_file_name.is_some() {
            authored_dirty_keys(&dirty, slot)?
        } else {
            dirty_keys(&dirty, slot)?
        };
        if let Some(config_name) = native_file_name {
            // CLI:2561,2615,2644,2684 stage `config_name` onto the preset's
            // settings ID before `update_full_config` (`:3099-3161`).
            system.insert(
                identity_key.to_owned(),
                RawConfigValue::Text(config_name.to_owned()),
            );
        }
        // `load_default_gcodes_to_config` (CLI:671–709,3239,3389) supplies
        // defaults for absent machine scripts in nonempty Native config;
        // TYPE_PRINT adds none.
        let defaultable_scripts =
            if skip_gcodes && kind == ProfileKind::Machine && system.entries().next().is_some() {
                MACHINE_GCODES
            } else {
                &[]
            };
        for &key in defaultable_scripts {
            if protected.contains(key)
                && !system.contains_key(key)
                && let Some(value) = keys::default_value(key)?
            {
                system.insert(key.to_owned(), value);
            }
        }
        let mut released = false;
        for (key, value) in system.entries() {
            let skip_bookkeeping =
                bookkeeping(key) && !(native_file_name.is_some() && key == identity_key);
            if skip_bookkeeping || connection(key) {
                continue;
            }
            if skip_gcodes && is_gcode_key(key) {
                released |= protected.remove(key);
            }
            let kept = if native_file_name.is_some() {
                protected.contains(key.as_str())
            } else {
                project.contains_key(key) && (!refresh_unlisted || protected.contains(key.as_str()))
            };
            if kept {
                continue;
            }
            // CLI:3188–3193 compares loaded options, not archive strings.
            retained_thumbnails |= kind == ProfileKind::Machine && key == "thumbnails";
            result.insert(key.clone(), value.clone());
        }
        if released {
            // CLI:3244–3253,3394–3403 serialize the remaining authored set.
            dirty_out[slot] = dirty_columns::encode(protected.into_iter().collect())?;
            dirty_changed = true;
        }
    }
    // CLI:2295,2693,2793: a supplied named machine skips catalog filament refresh
    // (but still allows Native --uptodate-filaments / filament_full files);
    // explicit --load-filaments replaces instead of refreshing.
    let machine_replaced = supplied.is_some_and(|p| !p.printer_settings_id.0.is_empty());
    if supplied.is_some_and(|p| !p.filament_settings_id.0.is_empty())
        || (machine_replaced && matches!(lookup, PresetLookup::Catalog { .. }))
    {
        if dirty_changed {
            result.insert_strings("different_settings_to_system", &dirty_out);
        }
        return Ok((result, retained_thumbnails));
    }
    let mut filaments = names
        .iter()
        .enumerate()
        .map(|(slot, name)| {
            let name = system_name(name, &inherited, slot + 1);
            // CLI:2805–2807: a resolver miss keeps the imported filament slot.
            if name.is_empty() {
                Ok(None)
            } else if machine_replaced {
                Ok(lookup.native_file(ProfileKind::Filament, slot))
            } else {
                lookup.system(ProfileKind::Filament, name, slot)
            }
        })
        .collect::<Result<Vec<_>, SliceError>>()?;
    filament::apply(
        &mut result,
        project,
        &mut filaments,
        (&dirty, &mut dirty_out, &mut dirty_changed),
        (refresh_unlisted, lookup.skip_modified_gcodes()),
    )?;
    if dirty_changed {
        result.insert_strings("different_settings_to_system", &dirty_out);
    }
    Ok((result, retained_thumbnails))
}

pub(super) fn system_config(
    catalog: &Catalog,
    kind: ProfileKind,
    name: &str,
) -> Result<Option<ProjectSettingsRaw>, SliceError> {
    let Some(position) = catalog
        .selectable_position(kind, name)
        .filter(|position| !catalog.is_user_scope(*position))
    else {
        return Ok(None);
    };
    let membership = keys::names(kind).collect::<BTreeSet<_>>();
    let mut raw = ProjectSettingsRaw::default();
    raw.extend(super::default_preset_fields(kind)?);
    let vendor = !catalog.is_default(position);
    for layer in catalog.resolved_layers(position)? {
        let mut fragment = ProjectSettingsRaw::default();
        fragment.extend(
            layer
                .fields
                .iter()
                .map(|(key, value)| (key.clone(), value.clone())),
        );
        fragment.handle_legacy();
        if kind == ProfileKind::Machine {
            normalize_authored_thumbnails(&mut fragment)?;
        }
        if vendor {
            fragment
                .normalize_preset_string_vectors()
                .map_err(SliceError::InvalidInput)?;
            preset_normalization::replace_nil_from_defaults(&mut fragment, &raw)?;
        }
        raw.extend(
            fragment
                .into_entries()
                .into_iter()
                .filter(|(key, _)| membership.contains(key.as_str())),
        );
        // PresetBundle.cpp:7839–7848 completes each vendor config before it can
        // become the parent of another preset or be adopted by the CLI merge.
        if vendor {
            preset_normalization::extend_default_config_length(&mut raw)?;
        }
    }
    raw.normalize_wire_vector_shapes()
        .map_err(SliceError::InvalidInput)?;
    if kind == ProfileKind::Machine {
        preset_normalization::normalize_machine(&mut raw)?;
    }
    Ok(Some(raw))
}

// Config.cpp:1149 converts authored JSON before preset defaults and inheritance.
pub(super) fn normalize_authored_thumbnails(
    raw: &mut ProjectSettingsRaw,
) -> Result<(), SliceError> {
    let Some(value) = raw.get("thumbnails") else {
        return Ok(());
    };
    let mut authored = ProjectSettingsRaw::default();
    authored.insert("thumbnails".to_owned(), value.clone());
    if let Some(format) = raw.get("thumbnails_format") {
        authored.insert("thumbnails_format".to_owned(), format.clone());
    }
    authored
        .normalize_scalar_arrays()
        .and_then(|()| authored.normalize_loaded_composites())
        .map_err(SliceError::InvalidInput)?;
    raw.insert(
        "thumbnails".to_owned(),
        authored.get("thumbnails").unwrap().clone(),
    );
    Ok(())
}

pub(super) fn system_name<'a>(name: &'a str, inherited: &'a [String], slot: usize) -> &'a str {
    inherited
        .get(slot)
        .map(String::as_str)
        .filter(|value| !value.is_empty())
        .unwrap_or(name)
}

/// `OrcaSlicer.cpp:1899-1909,2563-2593`: the three dimensions used by the
/// retained-machine admission, with the same bounding-box and integer truncation.
pub(super) fn printable_size(raw: &ProjectSettingsRaw) -> [i32; 3] {
    let points = raw
        .get("printable_area")
        .map(point_pairs)
        .unwrap_or_default();
    let (width, depth) = if points.len() >= 4 {
        let (min_x, max_x, min_y, max_y) = points.iter().skip(1).fold(
            (points[0].0, points[0].0, points[0].1, points[0].1),
            |(min_x, max_x, min_y, max_y), &(x, y)| {
                (min_x.min(x), max_x.max(x), min_y.min(y), max_y.max(y))
            },
        );
        ((max_x - min_x) as i32, (max_y - min_y) as i32)
    } else {
        (0, 0)
    };
    let height = raw
        .get("printable_height")
        .and_then(|value| value.token())
        .and_then(|value| value.trim().parse::<f64>().ok())
        .map_or(0, |value| value as i32);
    [width, depth, height]
}

fn point_pairs(value: &RawConfigValue) -> Vec<(f64, f64)> {
    let tokens: Vec<String> = match value {
        RawConfigValue::List(values) => values.iter().filter_map(|value| value.token()).collect(),
        RawConfigValue::Text(value) => value.split(';').map(str::to_owned).collect(),
        _ => Vec::new(),
    };
    tokens
        .iter()
        .filter_map(|token| {
            let (x, y) = token.trim().split_once('x')?;
            Some((x.trim().parse().ok()?, y.trim().parse().ok()?))
        })
        .collect()
}

pub(in crate::profiles) fn strings(raw: &ProjectSettingsRaw, key: &str) -> Vec<String> {
    let Some(value) = raw.get(key) else {
        return Vec::new();
    };
    match value {
        RawConfigValue::List(values) => values
            .iter()
            .map(|value| match value {
                RawConfigValue::Text(text) => text.clone(),
                other => other.token().unwrap_or_default(),
            })
            .collect(),
        RawConfigValue::Text(text) => vec![text.clone()],
        _ => Vec::new(),
    }
}

pub(super) fn authored_dirty_keys(
    dirty: &[String],
    slot: usize,
) -> Result<BTreeSet<String>, SliceError> {
    let token = dirty.get(slot).map(String::as_str).unwrap_or("");
    crate::options::unescape_strings_cstyle(token)
        .map(|keys| keys.into_iter().collect())
        .map_err(SliceError::InvalidInput)
}

pub(in crate::profiles) fn dirty_keys(
    dirty: &[String],
    slot: usize,
) -> Result<BTreeSet<String>, SliceError> {
    let mut keys = authored_dirty_keys(dirty, slot)?;
    // PresetBundle::project_different_keys: identity is never refreshed.
    keys.extend(
        [
            "inherits",
            "print_settings_id",
            "filament_settings_id",
            "printer_settings_id",
        ]
        .map(str::to_owned),
    );
    Ok(keys)
}

pub(super) fn bookkeeping(key: &str) -> bool {
    matches!(
        key,
        "inherits"
            | "compatible_printers"
            | "compatible_prints"
            | "compatible_printers_condition"
            | "compatible_prints_condition"
            | "print_settings_id"
            | "printer_settings_id"
    )
}

pub(crate) fn connection(key: &str) -> bool {
    matches!(
        key,
        "print_host"
            | "print_host_webui"
            | "printhost_apikey"
            | "printhost_cafile"
            | "printhost_user"
            | "printhost_password"
            | "printhost_port"
    )
}
