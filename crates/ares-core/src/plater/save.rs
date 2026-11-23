use super::{NO_EDITS, PlaterSession};
use crate::project::export::write_plater_project;
use crate::project::raw_settings::ProjectSettingsRaw;
use crate::{ConfigOverrides, Placement, ProfileRequest, ProfileSliceRequest, SliceError};

impl PlaterSession {
    /// Saves the edited model and its project presets without slicing any plate.
    pub fn save_project(&self, overrides: &ConfigOverrides) -> Result<Vec<u8>, SliceError> {
        let (mut project, _) = self.source().load(overrides, None, None, (false, None))?;
        project.replay_slot_options(&self.slot_changes)?;
        self.save_with_config(project.documents().project_settings_raw.clone())
    }

    /// Saves the edited model with the same preset composition as a plater slice.
    pub fn save_project_with_profiles<'a>(
        &self,
        request: impl ProfileRequest<'a>,
    ) -> Result<Vec<u8>, SliceError> {
        let composition = request.composition();
        let request = ProfileSliceRequest {
            plate: None,
            placement: Placement::None,
            ..request.request()
        };
        crate::profiles::with_edited_profile_source(
            self.source(),
            (request, composition),
            &NO_EDITS,
            |source| {
                let mut project = crate::project_slice::load_profile_project(&source)?;
                if source.refresh.is_some() {
                    project.replay_slot_options(&self.slot_changes)?;
                }
                self.save_with_config(project.documents().project_settings_raw.clone())
            },
        )
    }

    fn save_with_config(&self, mut raw: ProjectSettingsRaw) -> Result<Vec<u8>, SliceError> {
        if let Some((x, y)) = &self.wipe_tower {
            for (key, list) in [("wipe_tower_x", x), ("wipe_tower_y", y)] {
                let values = list
                    .iter()
                    .map(|&value| crate::options::format_number(value))
                    .collect::<Vec<_>>();
                raw.insert_strings(key, &values);
            }
        }
        write_plater_project(self.project(), &raw, Some(&self.plates()))
    }
}
