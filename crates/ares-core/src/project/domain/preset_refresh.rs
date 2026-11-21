use super::Project;
use crate::{ConfigOverrides, SliceError, project::raw_settings::ProjectSettingsRaw};

impl Project {
    pub fn preset_selection(&self) -> Option<&crate::profiles::PresetSelection> {
        self.documents.preset_selection.as_deref()
    }

    pub(crate) fn apply_project_preset_selection(
        &mut self,
        raw: ProjectSettingsRaw,
        selection: crate::profiles::PresetSelection,
        overrides: &ConfigOverrides,
    ) -> Result<(), SliceError> {
        self.apply_project_preset_refresh(raw, overrides)?;
        self.documents.preset_selection = Some(std::sync::Arc::new(selection));
        self.documents.config_composer = super::documents::ConfigComposer::FullFffConfig;
        Ok(())
    }

    /// The explicit plater selection's config is full_fff_config's.
    pub(crate) fn compose_as_full_fff_config(&mut self) {
        self.documents.config_composer = super::documents::ConfigComposer::FullFffConfig;
    }

    pub(crate) fn apply_project_preset_refresh(
        &mut self,
        mut raw: ProjectSettingsRaw,
        overrides: &ConfigOverrides,
    ) -> Result<(), SliceError> {
        overrides.apply(&mut raw)?;
        raw.handle_legacy();
        raw.normalize_wire_vector_shapes()
            .map_err(SliceError::InvalidInput)?;
        let mut materialized_raw = raw.clone();
        materialized_raw.complete_filament_variant_defaults()?;
        self.settings = crate::options::json_deserialize::deserialize_refreshed_config(
            materialized_raw.into_object(),
        )
        .map_err(|error| {
            SliceError::InvalidInput(format!("refreshed project settings are invalid: {error}"))
        })?
        .resolve();
        self.documents.project_settings_raw = raw;
        self.initialize_cli_loaded_project_config(self.cli_filament_count());
        Ok(())
    }
}
