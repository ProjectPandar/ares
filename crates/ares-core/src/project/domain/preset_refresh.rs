use super::Project;
use crate::{
    ConfigOverrides, SliceError, options::ProjectSettingsBuilder,
    project::raw_settings::ProjectSettingsRaw,
};

impl Project {
    pub(crate) fn apply_project_preset_refresh(
        &mut self,
        mut raw: ProjectSettingsRaw,
        overrides: &ConfigOverrides,
    ) -> Result<(), SliceError> {
        overrides.apply(&mut raw)?;
        raw.handle_legacy();
        raw.normalize_wire_vector_shapes()
            .map_err(SliceError::InvalidInput)?;
        raw.complete_filament_variant_defaults()?;
        self.settings = crate::options::json_deserialize::deserialize_preset_config::<
            ProjectSettingsBuilder,
        >(raw.clone().into_object())
        .map_err(|error| {
            SliceError::InvalidInput(format!("refreshed project settings are invalid: {error}"))
        })?
        .resolve();
        self.documents.project_settings_raw = raw;
        self.initialize_cli_loaded_project_config(self.cli_filament_count());
        Ok(())
    }
}
