//! Embedded configuration retained by CLI's independent preset-kind loop
//! (`OrcaSlicer.cpp:1728–1760,2280–2374`). No mesh geometry is imported here.

use super::{LoadSettingsRequest, invalid};
use crate::{
    ConfigOverrides, InputFormat, OptionValueKind, Project, SliceError,
    options::{json_deserialize, registry::option_definition},
    project::{
        ArchiveLimits, PackagePath, ProjectArchive, SelectedProfile,
        raw_settings::{ProjectSettingsRaw, RawConfigValue},
    },
    project_slice::compatibility::LoadedPresetNames,
};

pub(crate) fn project_for_3mf_export(
    input: &[u8],
    request: &LoadSettingsRequest<'_>,
) -> Result<Project, SliceError> {
    super::validate_stl_export_presets(request)?;
    let (composed, names) = if request.settings.is_empty() && request.filaments.is_empty() {
        (
            None,
            LoadedPresetNames {
                machine: None,
                printer_model: String::new(),
                process: None,
                embedded: Some(read_embedded_config(input)?),
                direct_stl_machine: None,
                filament_indices: Vec::new(),
            },
        )
    } else {
        let (composed, names) = super::compose_project_load_settings((input, None), request)?;
        (Some(composed), names)
    };
    let mut default_settings = crate::ProjectSettings::default();
    default_settings
        .project
        .preset
        .printer_settings_id
        .0
        .clear();
    default_settings.project.preset.print_settings_id.0.clear();
    default_settings
        .project
        .preset
        .filament_settings_id
        .0
        .clear();
    default_settings.project.gcode.filament_ids.0.clear();
    let empty_raw = ProjectSettingsRaw::default();
    let profile = match &composed {
        Some(composed) => SelectedProfile {
            settings: composed.profile.settings(),
            profile_raw: &composed.raw,
            filament_raws: &composed.filament_raws,
            validate_post_process: true,
            enable_timelapse: false,
            load_names: Some(&names),
            retained_refresh: None,
            full_fff_config: None,
        },
        None => SelectedProfile {
            settings: &default_settings,
            profile_raw: &empty_raw,
            filament_raws: &[],
            validate_post_process: true,
            enable_timelapse: false,
            load_names: Some(&names),
            retained_refresh: None,
            full_fff_config: None,
        },
    };
    let (project, _) = Project::from_3mf_with_profiles(
        (input, None),
        profile,
        &ConfigOverrides::default(),
        (None, &crate::plater::NO_EDITS),
        None,
    )?;
    if names.machine.is_some() && names.process.is_none() && project.documents().bbl_family_3mf {
        let embedded = match &names.embedded {
            Some(config) => config.clone(),
            None => read_embedded_config(input)?,
        };
        let original = embedded
            .get("print_settings_id")
            .and_then(|value| value.element(0))
            .and_then(RawConfigValue::token)
            .unwrap_or_default();
        if !original.is_empty() && project.settings().project.preset.print_settings_id.0 != original
        {
            return Err(invalid(
                "unsupported 3MF export: machine-switch project-embedded preset".to_owned(),
            ));
        }
    }
    Ok(project)
}

pub(super) fn retained_config(
    (input, embedded_raw): (&[u8], Option<&ProjectSettingsRaw>),
    request: &LoadSettingsRequest<'_>,
    names: &LoadedPresetNames,
) -> Result<Option<ProjectSettingsRaw>, SliceError> {
    if InputFormat::detect(input) != InputFormat::ThreeMf
        || !request.filaments.iter().all(|file| file.json.is_empty())
        || (names.machine.is_none() && names.process.is_none())
    {
        return Ok(None);
    }
    match embedded_raw {
        Some(raw) => Ok(Some(raw.clone())),
        None => read_embedded_config(input).map(Some),
    }
}

fn read_embedded_config(input: &[u8]) -> Result<ProjectSettingsRaw, SliceError> {
    let mut archive = ProjectArchive::open(input, ArchiveLimits::PROJECT)?;
    let path = PackagePath::root().resolve("Metadata/project_settings.config")?;
    serde_json::from_slice(&archive.read(&path)?).map_err(|error| {
        SliceError::InvalidInput(format!("invalid embedded project settings: {error}"))
    })
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
