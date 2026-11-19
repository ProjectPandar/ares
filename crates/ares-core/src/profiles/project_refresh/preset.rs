use std::collections::BTreeSet;

use super::super::{ProfileKind, catalog::Catalog};
use super::{PresetLookup, filament, keys};
use crate::{
    SliceError,
    project::raw_settings::{ProjectSettingsRaw, RawConfigValue},
};

pub(super) fn refresh(
    lookup: PresetLookup<'_>,
    project: &ProjectSettingsRaw,
    uptodate: bool,
    selected: Option<&crate::ProjectSettings>,
) -> Result<ProjectSettingsRaw, SliceError> {
    let names = strings(project, "filament_settings_id");
    let inherited = strings(project, "inherits_group");
    let dirty = strings(project, "different_settings_to_system");
    let refresh_unlisted = uptodate || dirty.iter().any(|entry| !entry.is_empty());
    let mut result = project.clone();
    // CLI:3167–3200: machine and process refresh, not replacement selection.
    let supplied = selected.map(|selected| &selected.project.preset);
    for (kind, key, slot, replaced) in [
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
        if replaced {
            continue;
        }
        let identity = strings(project, key);
        let name = identity.first().map(String::as_str).unwrap_or("");
        let name = system_name(name, &inherited, slot);
        if name.is_empty() {
            continue;
        }
        // CLI:3182-3185: a resolver miss keeps the project's values.
        let Some(system) = lookup.system(kind, name, slot)? else {
            continue;
        };
        let protected = dirty_keys(&dirty, slot)?;
        for (key, value) in system.entries() {
            if bookkeeping(key)
                || connection(key)
                || (project.contains_key(key)
                    && (!refresh_unlisted || protected.contains(key.as_str())))
            {
                continue;
            }
            result.insert(key.clone(), value.clone());
        }
    }
    // CLI:2295,2793: a supplied named machine skips retained filament refresh.
    if supplied.is_some_and(|p| {
        (!p.printer_settings_id.0.is_empty() && matches!(lookup, PresetLookup::Catalog(_)))
            || !p.filament_settings_id.0.is_empty()
    }) {
        return Ok(result);
    }
    let filaments = names
        .iter()
        .enumerate()
        .map(|(slot, name)| {
            let name = system_name(name, &inherited, slot + 1);
            // CLI:2805–2807: a resolver miss keeps the imported filament slot.
            if name.is_empty() {
                Ok(None)
            } else {
                lookup.system(ProfileKind::Filament, name, slot)
            }
        })
        .collect::<Result<Vec<_>, SliceError>>()?;
    filament::apply(&mut result, project, &filaments, &dirty, refresh_unlisted)?;
    Ok(result)
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
    for layer in catalog.resolved_layers(position)? {
        let mut fragment = ProjectSettingsRaw::default();
        fragment.extend(
            layer
                .fields
                .iter()
                .map(|(key, value)| (key.clone(), value.clone())),
        );
        fragment.handle_legacy();
        for (key, value) in fragment.entries() {
            if membership.contains(key.as_str()) {
                raw.insert(key.clone(), value.clone());
            }
        }
    }
    raw.normalize_wire_vector_shapes()
        .map_err(SliceError::InvalidInput)?;
    if kind == ProfileKind::Machine {
        super::super::preset_normalization::normalize_machine(&mut raw)?;
    }
    Ok(Some(raw))
}

pub(super) fn system_name<'a>(name: &'a str, inherited: &'a [String], slot: usize) -> &'a str {
    inherited
        .get(slot)
        .map(String::as_str)
        .filter(|value| !value.is_empty())
        .unwrap_or(name)
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

pub(in crate::profiles) fn dirty_keys(
    dirty: &[String],
    slot: usize,
) -> Result<BTreeSet<String>, SliceError> {
    let token = dirty.get(slot).map(String::as_str).unwrap_or("");
    let mut keys = crate::options::unescape_strings_cstyle(token)
        .map_err(SliceError::InvalidInput)?
        .into_iter()
        .collect::<BTreeSet<_>>();
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
