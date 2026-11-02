mod filament;
mod keys;
mod preset;

use super::{Placement, catalog::Catalog};
use crate::{ConfigOverrides, Project, SliceError};

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
