use std::{fs, io, io::Write, path::Path};

use ares_core::{CliPresetProject, LoadSettingsFile, LoadSettingsRequest};

use crate::{
    args::DirectSliceArgs,
    failure::{CliFailure, substitution_warnings},
    preset_sources, split_native_list,
};

pub(super) fn run(
    input: &Path,
    destination: &Path,
    args: &DirectSliceArgs,
) -> Result<(), CliFailure> {
    let project = fs::read(input)?;
    if destination != Path::new("-")
        && let Some(settings) = args.load_settings.as_deref()
    {
        let settings = split_native_list(settings);
        let filaments = args.load_filaments.as_deref().map(split_native_list);
        if settings.len() == 2
            && settings.iter().all(|label| !label.is_empty())
            && filaments.as_ref().is_none_or(|labels| {
                labels.len() == 2 && labels.iter().all(|label| !label.is_empty())
            })
        {
            let context = args
                .project_actions
                .context(ares_core::SlicerContext::new(0));
            let admitted = CliPresetProject::admit(&project, &context).map_err(io::Error::other)?;
            if admitted.is_bbl_family_3mf() {
                let json = export_explicit(
                    admitted,
                    (&settings, filaments.as_deref().unwrap_or_default()),
                    args,
                )?;
                return report_and_write(input, &project, destination, &json);
            }
        }
    }
    let resources = crate::installed_resources::resolve(None)?;
    let catalog = crate::project_presets::load_resources_catalog(&resources)?;
    let json = ares_core::export_project_settings_with_catalog(&project, &catalog)
        .map_err(io::Error::other)?;
    report_and_write(input, &project, destination, &json)
}

fn export_explicit(
    project: CliPresetProject<'_>,
    (settings, filaments): (&[&str], &[&str]),
    args: &DirectSliceArgs,
) -> Result<Vec<u8>, CliFailure> {
    let settings = preset_sources::read_preset_files(settings)?;
    let filaments = preset_sources::read_preset_files(filaments)?;
    let [settings, filaments] = [&settings, &filaments].map(|files| {
        files
            .iter()
            .map(|(label, json, source_identity)| LoadSettingsFile {
                label,
                json,
                source_identity: source_identity.as_ref(),
            })
            .collect::<Vec<_>>()
    });
    let loaded_sources = preset_sources::load(
        args.datadir.as_deref(),
        settings.iter().chain(&filaments).copied(),
    )
    .map_err(|error| io::Error::other(error.to_string()))?;
    project
        .export_settings_with_load_settings(
            &LoadSettingsRequest {
                loaded_sources: Some(&loaded_sources),
                settings: &settings,
                filaments: &filaments,
            },
            args.enable_timelapse,
        )
        .map_err(|error| io::Error::other(error).into())
}

fn report_and_write(
    input: &Path,
    project: &[u8],
    destination: &Path,
    json: &[u8],
) -> Result<(), CliFailure> {
    let substitutions = crate::substitutions(project);
    eprint!(
        "{}",
        substitution_warnings(&input.to_string_lossy(), &substitutions)
    );
    write(destination, json)
}

pub(super) fn write(destination: &Path, json: &[u8]) -> Result<(), CliFailure> {
    if destination == Path::new("-") {
        io::stdout().lock().write_all(json)?;
    } else {
        // ConfigBase::save_to_json serializes before replacing the destination.
        let parent = destination
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let mut file = tempfile::NamedTempFile::new_in(parent)?;
        file.write_all(json)?;
        file.persist(destination).map_err(|error| error.error)?;
    }
    Ok(())
}
