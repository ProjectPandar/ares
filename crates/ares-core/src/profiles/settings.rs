use serde::{Serialize, Serializer, ser::SerializeMap};

use super::{PresetSelection, ProfileSliceRequest};
use crate::{SliceError, project::raw_settings::ProjectSettingsRaw};

/// Effective settings plus the reference/edited collection state of an Always request.
pub struct ProfileSettings {
    settings: ProjectSettingsRaw,
    selection: Option<std::sync::Arc<PresetSelection>>,
}

impl Serialize for ProfileSettings {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let Some(selection) = &self.selection else {
            return self.settings.serialize(serializer);
        };
        let mut result = serializer.serialize_map(Some(2))?;
        result.serialize_entry("settings", &self.settings)?;
        result.serialize_entry("preset_selection", selection.as_ref())?;
        result.end()
    }
}

/// The effective settings and selection state consumed by the profile slice entry.
pub async fn profile_settings(
    input: impl AsRef<[u8]>,
    request: ProfileSliceRequest<'_>,
) -> Result<ProfileSettings, SliceError> {
    super::entry::with_profile_source(input.as_ref(), request, |source| {
        let project = crate::project_slice::load_profile_project(&source)?;
        Ok(ProfileSettings {
            settings: project.documents().project_settings_raw.clone(),
            selection: project.documents().preset_selection.clone(),
        })
    })
}
