//! Embedded configuration retained by CLI's independent preset-kind loop
//! (`OrcaSlicer.cpp:1728–1760,2280–2374`). No mesh geometry is imported here.

use super::LoadSettingsRequest;
use crate::{
    InputFormat, SliceError,
    project::{ArchiveLimits, PackagePath, ProjectArchive, raw_settings::ProjectSettingsRaw},
    project_slice::compatibility::LoadedPresetNames,
};

pub(super) fn retained_config(
    input: &[u8],
    request: &LoadSettingsRequest<'_>,
    names: &LoadedPresetNames,
) -> Result<Option<ProjectSettingsRaw>, SliceError> {
    if InputFormat::detect(input) != InputFormat::ThreeMf
        || names.process.is_some()
        || !request.filaments.iter().all(|file| file.json.is_empty())
        || !names
            .machine
            .as_ref()
            .is_some_and(|machine| machine.inherits.as_deref().is_none_or(str::is_empty))
    {
        return Ok(None);
    }
    let mut archive = ProjectArchive::open(input, ArchiveLimits::PROJECT)?;
    let path = PackagePath::root().resolve("Metadata/project_settings.config")?;
    let config = serde_json::from_slice(&archive.read(&path)?).map_err(|error| {
        SliceError::InvalidInput(format!("invalid embedded project settings: {error}"))
    })?;
    Ok(Some(config))
}
