use std::{fs, path::Path};

use ares_core::{LoadSettingsFile, LoadSettingsRequest};

use crate::{CliFailure, SliceArgs, read_preset_files, split_native_list};

pub(super) async fn run(args: &SliceArgs, output: &Path) -> Result<(), CliFailure> {
    let output = args.outputdir.as_ref().map_or_else(
        || output.to_owned(),
        |dir| {
            // Native concatenates the string rather than treating an absolute
            // export filename as overriding --outputdir.
            let mut path = dir.as_os_str().to_owned();
            path.push("/");
            path.push(output);
            path.into()
        },
    );
    if args.slice_for_export == Some(1) {
        return run_sliced(args, &output).await;
    }
    let unsupported = [
        (args.slice_for_export.is_some(), "--slice"),
        (args.min_save.is_some_and(|value| value != 0), "--min-save"),
        (args.load_slicedata.is_some(), "--load-slicedata"),
        (args.export_slicedata.is_some(), "--export-slicedata"),
        (args.arrange.is_some_and(|value| value != 0), "--arrange"),
        (args.orient != 0, "--orient"),
        (args.plate.is_some(), "--plate"),
        (args.dump_moves.is_some(), "--dump-moves"),
        (args.overrides.is_some(), "--overrides"),
        (args.options.is_some(), "--options"),
        (args.output_path.is_some(), "-o"),
        (
            args.project_preset_catalog.is_some(),
            "--project-preset-catalog",
        ),
        (args.resources_dir.is_some(), "--resources-dir"),
        (args.progress, "--progress"),
    ];
    if let Some((_, option)) = unsupported.into_iter().find(|(present, _)| *present) {
        return Err(CliFailure::project_export(
            &output,
            true,
            format!("unsupported 3MF export option {option}"),
        ));
    }
    let failed = |detail| CliFailure::project_export(&output, false, detail);
    let input = match fs::read(&args.input) {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Err(CliFailure::missing_input(
                &args.input,
                #[cfg(target_os = "linux")]
                &output,
            ));
        }
        Err(err) => return Err(failed(err.to_string())),
    };
    let filename = args
        .input
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| failed("invalid source filename".to_owned()))?;
    let settings = read_preset_files(
        &args
            .load_settings
            .as_deref()
            .map_or_else(Vec::new, split_native_list),
    )?;
    let filaments = read_preset_files(
        &args
            .load_filaments
            .as_deref()
            .map_or_else(Vec::new, split_native_list),
    )?;
    let bytes = ares_core::export_3mf(
        &input,
        filename,
        LoadSettingsRequest {
            settings: &sources(&settings),
            filaments: &sources(&filaments),
        },
    )
    .map_err(|err| failed(err.to_string()))?;
    fs::write(&output, bytes).map_err(|err| failed(err.to_string()))
}

async fn run_sliced(args: &SliceArgs, output: &Path) -> Result<(), CliFailure> {
    if args.outputdir.is_some() {
        return Err(CliFailure::project_export(
            output,
            true,
            "unsupported 3MF export option --outputdir".to_owned(),
        ));
    }
    let input = fs::read(&args.input).map_err(|err| {
        if err.kind() == std::io::ErrorKind::NotFound {
            CliFailure::missing_input(
                &args.input,
                #[cfg(target_os = "linux")]
                output,
            )
        } else {
            CliFailure::project_export(output, false, err.to_string())
        }
    })?;
    let filename = args
        .input
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            CliFailure::project_export(output, true, "invalid source filename".to_owned())
        })?;
    let overrides = ares_core::ConfigOverrides::default();
    let processed = if args
        .input
        .extension()
        .and_then(|extension| extension.to_str())
        == Some("stl")
    {
        let settings = read_preset_files(
            &args
                .load_settings
                .as_deref()
                .map_or_else(Vec::new, split_native_list),
        )?;
        let filaments = read_preset_files(
            &args
                .load_filaments
                .as_deref()
                .map_or_else(Vec::new, split_native_list),
        )?;
        let settings = settings
            .iter()
            .map(|(label, json)| LoadSettingsFile { label, json })
            .collect::<Vec<_>>();
        let filaments = filaments
            .iter()
            .map(|(label, json)| LoadSettingsFile { label, json })
            .collect::<Vec<_>>();
        let mut sliced_plate = ares_core::SlicedPlate::default();
        ares_core::slice_with_load_settings_archive(
            &input,
            ares_core::LoadSettingsSliceRequest {
                filename,
                catalog_json: None,
                uptodate: false,
                settings: &settings,
                filaments: &filaments,
                plate: None,
                overrides: &overrides,
                placement: ares_core::Placement::Center,
                orient: false,
                sliced_plate: &mut sliced_plate,
            },
            ares_core::GenerationMetadata::default(),
            ares_core::SlicerContext::new(0),
        )
        .await
        .map_err(|err| CliFailure::project_export(output, false, err.to_string()))?
    } else if args
        .input
        .extension()
        .and_then(|extension| extension.to_str())
        == Some("3mf")
    {
        ares_core::slice_project_plate_with_archive(
            &input,
            ares_core::GenerationMetadata::default(),
            Some(1),
            &overrides,
            ares_core::SlicerContext::new(0),
        )
        .await
        .map_err(|err| CliFailure::project_export(output, false, err.to_string()))?
    } else {
        return Err(CliFailure::project_export(
            output,
            true,
            "sliced export requires STL or 3MF input".to_owned(),
        ));
    };
    let bytes = ares_core::export_sliced_3mf(&input, filename, processed)
        .map_err(|err| CliFailure::project_export(output, false, err.to_string()))?;
    fs::write(output, bytes)
        .map_err(|err| CliFailure::project_export(output, false, err.to_string()))
}

fn sources(files: &[(String, String)]) -> Vec<LoadSettingsFile<'_>> {
    files
        .iter()
        .map(|(label, json)| LoadSettingsFile { label, json })
        .collect()
}
