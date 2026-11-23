//! Embedded configuration retained by CLI's independent preset-kind loop
//! (`OrcaSlicer.cpp:1728–1760,2280–2374`). No mesh geometry is imported here.

use super::{LoadSettingsRequest, invalid};
use crate::{
    InputFormat, OptionValueKind, SliceError,
    options::{json_deserialize, registry::option_definition},
    project::{
        ArchiveLimits, PackagePath, ProjectArchive,
        raw_settings::{ProjectSettingsRaw, RawConfigValue},
    },
    project_slice::compatibility::LoadedPresetNames,
};

pub(super) fn retained_config(
    input: &[u8],
    request: &LoadSettingsRequest<'_>,
    names: &LoadedPresetNames,
) -> Result<Option<ProjectSettingsRaw>, SliceError> {
    if InputFormat::detect(input) != InputFormat::ThreeMf
        || !request.filaments.iter().all(|file| file.json.is_empty())
        || (names.machine.is_none() && names.process.is_none())
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

/// CLI preset strings pass through `set_deserialize` (Config.cpp:987,
/// Config.hpp:1229) before `CLI::run` copies their first element into a slot.
pub(super) fn decode_filament_string_vectors(
    filaments: &mut [ProjectSettingsRaw],
) -> Result<(), SliceError> {
    for raw in filaments {
        let decoded = raw
            .entries()
            .filter_map(|(key, value)| {
                let RawConfigValue::Text(token) = value else {
                    return None;
                };
                option_definition(key)
                    .is_some_and(|definition| definition.kind == OptionValueKind::Strings)
                    .then(|| {
                        json_deserialize::decode(OptionValueKind::Strings, token)
                            .map(|value| (key.clone(), value))
                            .map_err(invalid)
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        raw.extend(decoded);
    }
    Ok(())
}
