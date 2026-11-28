use std::{fs, io, io::Write, path::Path};

use ares_core::{CliPresetProject, LoadSettingsRequest, SliceError};

use crate::{
    args::DirectSliceArgs,
    failure::{CliFailure, slice_error_to_failure},
    preset_sources::{self, as_load_settings_files},
    split_native_list,
};

pub(super) fn run(
    input: &Path,
    destination: &Path,
    args: &DirectSliceArgs,
    warnings: &mut crate::PositionalWarnings,
) -> Result<(), CliFailure> {
    let report = args
        .outputdir
        .as_deref()
        .unwrap_or_else(|| Path::new("."))
        .join("result.json");
    let project = fs::read(input)?;
    let opened = CliPresetProject::open(&project, args.allow_newer_file).map_err(|error| {
        if matches!(error, SliceError::FileVersionNotSupported) {
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
    let filaments = args.load_filaments.as_deref().map(split_native_list);
    let settings = args
        .load_settings
        .as_deref()
        .filter(|_| destination != Path::new("-"))
        .map(split_native_list)
        .filter(|labels| labels.len() == 2 && labels.iter().all(|label| !label.is_empty()))
        .filter(|_| {
            filaments.as_ref().is_none_or(|labels| {
                labels.len() == 2 && labels.iter().all(|label| !label.is_empty())
            })
        });
    let context = args
        .project_actions
        .context(ares_core::SlicerContext::new(0));
    let post_process = opened
        .check_post_process(&context)
        .map_err(io::Error::other);
    if post_process.is_ok() {
        warnings.record(input, &report, opened.substitutions());
    }
    if settings.is_some() {
        post_process?;
    }
    let json = if let Some(settings) = settings.filter(|_| opened.is_bbl_family_3mf()) {
        export_explicit(
            opened,
            (&settings, filaments.as_deref().unwrap_or_default()),
            args,
        )?
    } else {
        let resources = crate::installed_resources::resolve(None)?;
        let catalog = crate::project_presets::load_resources_catalog(&resources)?;
        opened
            .export_settings_with_catalog(&catalog)
            .map_err(io::Error::other)?
    };
    write(destination, &json)?;
    #[cfg(target_os = "linux")]
    crate::result_json::write_action_result(
        &report,
        (0, "Success."),
        (0, args.strict),
        &[],
        &warnings.take(),
    );
    Ok(())
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
