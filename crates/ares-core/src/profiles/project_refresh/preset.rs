use std::collections::BTreeSet;

use super::super::{ProfileKind, catalog::Catalog};
use super::{filament, keys};
use crate::{
    SliceError,
    project::raw_settings::{ProjectSettingsRaw, RawConfigValue},
};

pub(super) fn refresh(
    catalog: &Catalog,
    project: &ProjectSettingsRaw,
    uptodate: bool,
) -> Result<ProjectSettingsRaw, SliceError> {
    let names = strings(project, "filament_settings_id");
    let inherited = strings(project, "inherits_group");
    let dirty = strings(project, "different_settings_to_system");
    let refresh_unlisted = uptodate || dirty.iter().any(|entry| !entry.is_empty());
    let mut result = project.clone();
    // CLI:3167–3200: machine and process refresh, not replacement selection.
    for (kind, key, slot) in [
        (ProfileKind::Machine, "printer_settings_id", names.len() + 1),
        (ProfileKind::Process, "print_settings_id", 0),
    ] {
        let identity = strings(project, key);
        let name = identity.first().map(String::as_str).unwrap_or("");
        let name = system_name(name, &inherited, slot);
        if name.is_empty() {
            continue;
        }
        let system = system_config(catalog, kind, name)?;
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
    let filaments = names
        .iter()
        .enumerate()
        .map(|(slot, name)| {
            let name = system_name(name, &inherited, slot + 1);
            // CLI:2805–2807: a resolver miss keeps the imported filament slot.
            if name.is_empty()
                || catalog
                    .selectable_position(ProfileKind::Filament, name)
                    .is_none_or(|position| catalog.is_user_scope(position))
            {
                Ok(None)
            } else {
                system_config(catalog, ProfileKind::Filament, name).map(Some)
            }
        })
        .collect::<Result<Vec<_>, SliceError>>()?;
    filament::apply(&mut result, project, &filaments, &dirty, refresh_unlisted)?;
    Ok(result)
}

fn system_config(
    catalog: &Catalog,
    kind: ProfileKind,
    name: &str,
) -> Result<ProjectSettingsRaw, SliceError> {
    let position = catalog
        .selectable_position(kind, name)
        .filter(|position| !catalog.is_user_scope(*position))
        .ok_or_else(|| {
            SliceError::InvalidInput(format!("system preset '{name}' was not supplied"))
        })?;
    let membership = keys::names(kind).collect::<BTreeSet<_>>();
    let mut raw = ProjectSettingsRaw::default();
    for key in &membership {
        if let Some(value) = keys::default_value(key)? {
            raw.insert((*key).to_owned(), value);
        }
    }
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
    Ok(raw)
}

fn system_name<'a>(name: &'a str, inherited: &'a [String], slot: usize) -> &'a str {
    inherited
        .get(slot)
        .map(String::as_str)
        .filter(|value| !value.is_empty())
        .unwrap_or(name)
}

pub(super) fn strings(raw: &ProjectSettingsRaw, key: &str) -> Vec<String> {
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

pub(super) fn dirty_keys(dirty: &[String], slot: usize) -> Result<BTreeSet<String>, SliceError> {
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

fn bookkeeping(key: &str) -> bool {
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

fn connection(key: &str) -> bool {
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
