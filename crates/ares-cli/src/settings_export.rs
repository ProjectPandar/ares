use std::{fs, io, io::Write, path::Path};

use ares_core::{CliPresetProject, ConfigSubstitution, LoadSettingsRequest, SliceError};

use crate::{
    args::DirectSliceArgs,
    failure::{CliFailure, slice_error_to_failure, substitution_warnings},
    preset_sources::{self, as_load_settings_files},
    split_native_list,
};

pub(super) fn run(
    input: &Path,
    destination: &Path,
    args: &DirectSliceArgs,
) -> Result<(), CliFailure> {
    let project = fs::read(input)?;
    let opened = CliPresetProject::open(&project, args.allow_newer_file).map_err(|error| {
        if matches!(error, SliceError::FileVersionNotSupported) {
            let report = args
                .outputdir
                .as_deref()
                .unwrap_or_else(|| Path::new("."))
                .join("result.json");
            slice_error_to_failure(
                error,
                &report,
                #[cfg(target_os = "linux")]
                0,
            )
        } else {
            io::Error::other(error).into()
        }
    })?;
    let substitutions = opened.substitutions().to_vec();
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
            opened
                .check_post_process(&context)
                .map_err(io::Error::other)?;
            if opened.is_bbl_family_3mf() {
                let json = export_explicit(
                    opened,
                    (&settings, filaments.as_deref().unwrap_or_default()),
                    args,
                )?;
                return report_and_write(input, &substitutions, destination, &json);
            }
        }
    }
    let resources = crate::installed_resources::resolve(None)?;
    let catalog = crate::project_presets::load_resources_catalog(&resources)?;
    let json = opened
        .export_settings_with_catalog(&catalog)
        .map_err(io::Error::other)?;
    report_and_write(input, &substitutions, destination, &json)
}

fn export_explicit(
    project: CliPresetProject<'_>,
    (settings, filaments): (&[&str], &[&str]),
    args: &DirectSliceArgs,
) -> Result<Vec<u8>, CliFailure> {
    let settings_files = preset_sources::read_preset_files(settings)?;
    let filament_files = preset_sources::read_preset_files(filaments)?;
    let settings = as_load_settings_files(&settings_files);
    let filaments = as_load_settings_files(&filament_files);
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
    substitutions: &[ConfigSubstitution],
    destination: &Path,
    json: &[u8],
) -> Result<(), CliFailure> {
    eprint!(
        "{}",
        substitution_warnings(&input.to_string_lossy(), substitutions)
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
