mod filament;
pub(in crate::profiles) mod keys;
pub(super) mod preset;

use std::collections::BTreeMap;

use super::{Placement, project_selection::LoadedCatalog};
use crate::{
    ConfigOverrides, Project, ProjectSettings, SliceError,
    project::raw_settings::{ProjectSettingsRaw, RawConfigValue},
};

pub(super) fn default_preset_fields(
    kind: super::ProfileKind,
) -> Result<BTreeMap<String, RawConfigValue>, SliceError> {
    let mut fields = BTreeMap::new();
    for key in keys::names(kind) {
        if let Some(value) = keys::default_value(key)? {
            fields.insert(key.to_owned(), value);
        }
    }
    Ok(fields)
}

/// Preset.cpp: source-owned print, filament and printer option membership.
pub(crate) fn preset_owns_option(kind: super::ProfileKind, key: &str) -> bool {
    keys::names(kind).any(|name| name == key)
}

/// CLI:3167-3200 applies the collection-default keyset for retained project presets.
pub(crate) fn apply_retained_preset_defaults(
    raw: &mut ProjectSettingsRaw,
    selected: Option<&ProjectSettings>,
) -> Result<(), SliceError> {
    let inherited = preset::strings(raw, "inherits_group");
    let filament_count = preset::strings(raw, "filament_settings_id").len();
    let selection = selected.map(|settings| &settings.project.preset);
    for (kind, identity, loaded, slot) in [
        (
            super::ProfileKind::Machine,
            "printer_settings_id",
            selection.map_or("", |preset| preset.printer_settings_id.0.as_str()),
            filament_count + 1,
        ),
        (
            super::ProfileKind::Process,
            "print_settings_id",
            selection.map_or("", |preset| preset.print_settings_id.0.as_str()),
            0,
        ),
    ] {
        let names = preset::strings(raw, identity);
        let name = names.first().map(String::as_str).unwrap_or("");
        if !loaded.is_empty() || preset::system_name(name, &inherited, slot).is_empty() {
            continue;
        }
        for key in keys::names(kind) {
            let connection = preset::connection(key);
            if preset::bookkeeping(key)
                || (connection && selected.is_some())
                || (raw.contains_key(key) && !connection)
            {
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
pub(crate) enum ProjectPresetPolicy {
    Cli { uptodate: bool },
    Always,
}

#[derive(Clone, Copy)]
pub(crate) struct ProjectPresetRefresh<'a> {
    pub(super) catalog: &'a LoadedCatalog,
    pub(super) policy: ProjectPresetPolicy,
    pub(super) filename: &'a str,
    pub(super) placement: Placement,
}

pub(super) fn selection(input: &str) -> Result<Option<ProjectPresetPolicy>, SliceError> {
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Derived {
        from_project: bool,
        uptodate: Option<bool>,
        load_and_select: Option<LoadAndSelect>,
    }
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum LoadAndSelect {
        Always,
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
    let policy = match (derived.load_and_select, derived.uptodate) {
        (Some(LoadAndSelect::Always), None) => ProjectPresetPolicy::Always,
        (None, uptodate) => ProjectPresetPolicy::Cli {
            uptodate: uptodate.unwrap_or(false),
        },
        (Some(_), Some(_)) => {
            return Err(SliceError::InvalidInput(
                "uptodate and load_and_select are exclusive".to_owned(),
            ));
        }
    };
    Ok(Some(policy))
}

impl ProjectPresetRefresh<'_> {
    /// The plater's selection when it opens a project (`Plater.cpp:9622`), not a CLI path.
    pub(crate) fn is_plater(self) -> bool {
        matches!(self.policy, ProjectPresetPolicy::Always)
    }

    pub(crate) fn retained(
        self,
        raw: &ProjectSettingsRaw,
        selected: &ProjectSettings,
    ) -> Result<ProjectSettingsRaw, SliceError> {
        let uptodate = match self.policy {
            ProjectPresetPolicy::Cli { uptodate } => uptodate,
            ProjectPresetPolicy::Always => false,
        };
        preset::refresh(&self.catalog.catalog, raw, uptodate, Some(selected))
    }

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
        let (mut project, preparation) = crate::project::load_project_for_profiles(input)?;
        if validate {
            crate::project_slice::validate_post_process(&project)?;
        }
        match self.policy {
            ProjectPresetPolicy::Always => {
                let (raw, selection) =
                    super::project_selection::refresh(self.catalog, &project, self.filename)?;
                project.apply_project_preset_selection(raw, selection, overrides)?;
            }
            ProjectPresetPolicy::Cli { uptodate } => {
                let mut raw = preset::refresh(
                    &self.catalog.catalog,
                    &project.documents().project_settings_raw,
                    uptodate,
                    None,
                )?;
                // CLI:3167-3200 fills initially missing keys from the resolved system
                // preset before collection defaults can obscure their absence.
                apply_retained_preset_defaults(&mut raw, None)?;
                project.apply_project_preset_refresh(raw, overrides)?;
            }
        }
        Ok((project, preparation.for_overrides(overrides)))
    }

    pub(crate) fn place(self, project: &mut Project) {
        if self.placement == Placement::Center {
            super::super::project_slice::center_profile_project(project);
        }
    }
}
