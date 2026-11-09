mod filament;
mod keys;
mod preset;

use super::{Placement, catalog::Catalog};
use crate::{
    ConfigOverrides, Project, ProjectSettings, SliceError,
    project::raw_settings::ProjectSettingsRaw,
};

/// Preset.cpp: source-owned print, filament and printer option membership.
pub(crate) fn preset_owns_option(kind: super::ProfileKind, key: &str) -> bool {
    keys::names(kind).any(|name| name == key)
}

/// CLI:3167-3200 applies the collection-default keyset for retained project presets.
pub(crate) fn apply_retained_preset_defaults(
    raw: &mut ProjectSettingsRaw,
    selected: &ProjectSettings,
) -> Result<(), SliceError> {
    let inherited = preset::strings(raw, "inherits_group");
    let filament_count = preset::strings(raw, "filament_settings_id").len();
    let selection = &selected.project.preset;
    for (kind, identity, loaded, slot) in [
        (
            super::ProfileKind::Machine,
            "printer_settings_id",
            selection.printer_settings_id.0.as_str(),
            filament_count + 1,
        ),
        (
            super::ProfileKind::Process,
            "print_settings_id",
            selection.print_settings_id.0.as_str(),
            0,
        ),
    ] {
        let names = preset::strings(raw, identity);
        let name = names.first().map(String::as_str).unwrap_or("");
        if !loaded.is_empty() || preset::system_name(name, &inherited, slot).is_empty() {
            continue;
        }
        for key in keys::names(kind) {
            if raw.contains_key(key) || preset::bookkeeping(key) || preset::connection(key) {
                continue;
            }
            if let Some(value) = keys::default_value(key)? {
                raw.insert(key.to_owned(), value);
            }
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
pub(crate) struct ProjectPresetRefresh<'a> {
    pub(super) catalog: &'a Catalog,
    pub(super) uptodate: bool,
    pub(super) placement: Placement,
}

pub(super) fn selection(input: &str) -> Result<Option<bool>, SliceError> {
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Derived {
        from_project: bool,
        #[serde(default)]
        uptodate: bool,
    }
    let fields: std::collections::BTreeMap<String, crate::project::raw_settings::RawConfigValue> =
        serde_json::from_str(input).map_err(|error| {
            SliceError::InvalidInput(format!("invalid selection JSON: {error}"))
        })?;
    if !fields.contains_key("from_project") {
        return Ok(None);
    }
    let derived: Derived = serde_json::from_str(input).map_err(|error| {
        SliceError::InvalidInput(format!(
            "project-derived and explicit selections are exclusive: {error}"
        ))
    })?;
    if !derived.from_project {
        return Err(SliceError::InvalidInput(
            "from_project must be true".to_owned(),
        ));
    }
    Ok(Some(derived.uptodate))
}

impl ProjectPresetRefresh<'_> {
    pub(crate) fn load(
        self,
        input: &[u8],
        overrides: &ConfigOverrides,
        validate: bool,
    ) -> Result<(Project, crate::options::flush_matrix::PreparationContext), SliceError> {
        if crate::InputFormat::detect(input) != crate::InputFormat::ThreeMf {
            return Err(SliceError::InvalidInput(
                "project-derived selection requires a 3MF project".to_owned(),
            ));
        }
        let (mut project, preparation) =
            crate::project::load_project_with_preparation(input, &ConfigOverrides::default())?;
        if validate {
            crate::project_slice::validate_post_process(&project)?;
        }
        let raw = preset::refresh(
            self.catalog,
            &project.documents().project_settings_raw,
            self.uptodate,
        )?;
        project.apply_project_preset_refresh(raw, overrides)?;
        Ok((project, preparation.for_overrides(overrides)))
    }

    pub(crate) fn place(self, project: &mut Project) {
        if self.placement == Placement::Center {
            super::super::project_slice::center_profile_project(project);
        }
    }
}
