use std::{fs, path::Path};

use ares_core::{LoadSettingsFile, LoadSettingsRequest, SliceError};

use crate::{CliFailure, SliceArgs, failure::StlExportFailure};

pub(super) fn run(args: &SliceArgs) -> Result<(), CliFailure> {
    let outputdir = args.outputdir.as_deref().unwrap_or_else(|| Path::new("."));
    let report_path = outputdir.join("result.stl");
    let failure = |kind, detail| {
        CliFailure::stl_export(
            #[cfg(target_os = "linux")]
            &report_path,
            kind,
            detail,
        )
    };
    let unsupported = [
        (
            args.export_stl.is_none() && args.export_stls.is_none(),
            "--repair without an STL export action",
        ),
        (
            args.export_stl.is_some() && args.export_stls.is_some(),
            "--export-stl with --export-stls",
        ),
        (args.export_3mf.is_some(), "--export-3mf"),
        (args.slice_for_export.is_some(), "--slice"),
        (args.min_save.is_some(), "--min-save"),
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
        return Err(failure(
            StlExportFailure::Unsupported,
            format!("unsupported STL export option {option}"),
        ));
    }
    let files = std::iter::once(&args.input)
        .chain(&args.additional_inputs)
        .map(|path| {
            let bytes = fs::read(path).map_err(|err| match err.kind() {
                std::io::ErrorKind::NotFound => CliFailure::missing_input(
                    path,
                    #[cfg(target_os = "linux")]
                    &report_path,
                ),
                _ => failure(StlExportFailure::Input, err.to_string()),
            })?;
            let filename = path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| {
                    failure(
                        StlExportFailure::Unsupported,
                        "invalid source filename".to_owned(),
                    )
                })?;
            Ok((bytes, filename))
        })
        .collect::<Result<Vec<_>, CliFailure>>()?;
    let artifacts = ares_core::export_stls(
        files
            .iter()
            .map(|(bytes, filename)| (bytes.as_slice(), *filename)),
        args.repair.unwrap_or_default(),
    )
    .map_err(|err| match err {
        SliceError::UnsupportedProjectFeature(reason) => failure(
            StlExportFailure::Unsupported,
            format!("unsupported {reason}"),
        ),
        _ => failure(StlExportFailure::Input, err.to_string()),
    })?;
    validate_presets(args, &report_path)?;
    let dir = args
        .export_stls
        .as_deref()
        .filter(|path| !path.as_os_str().is_empty())
        .map_or_else(|| outputdir.join("stl"), Path::to_path_buf);
    let write_failed = |path: &Path, err| {
        failure(
            StlExportFailure::Output,
            format!("Model export to {} failed: {err}", path.display()),
        )
    };
    fs::create_dir_all(&dir).map_err(|err| write_failed(&dir, err))?;
    for artifact in artifacts {
        let path = dir.join(artifact.filename);
        fs::write(&path, artifact.bytes).map_err(|err| write_failed(&path, err))?;
    }
    #[cfg(target_os = "linux")]
    let _ = crate::result_json::ResultJson {
        return_code: 0,
        error_string: "Success.",
        plate_index: 0,
        layer_height: 0.0,
        wall_loops: 0,
        sparse_infill_density: 0.0,
        upward_machines: &[],
        sliced_plates: &[],
        warnings: &[],
    }
    .write(&report_path);
    Ok(())
}

fn validate_presets(args: &SliceArgs, report: &Path) -> Result<(), CliFailure> {
    let preset_error = |code, message, stderr| {
        CliFailure::export_failure(
            #[cfg(target_os = "linux")]
            report,
            (code, message),
            stderr,
        )
    };
    #[cfg(not(target_os = "linux"))]
    let _ = report;
    let invalid = |stderr| {
        preset_error(
            -5,
            "The input preset file is invalid and can not be parsed.",
            stderr,
        )
    };
    let read = |label: &str| {
        fs::read_to_string(label).map_err(|err| match err.kind() {
            std::io::ErrorKind::NotFound => preset_error(
                -3,
                "The input files to the slicer are not found.",
                format!("operator(): can not find setting file: {label}\n"),
            ),
            _ => invalid(String::new()),
        })
    };
    let settings = args
        .load_settings
        .as_deref()
        .into_iter()
        .flat_map(crate::split_native_list);
    let filaments = args
        .load_filaments
        .as_deref()
        .into_iter()
        .flat_map(crate::split_native_list)
        .filter(|label| !label.is_empty());
    let settings = settings
        .map(|label| read(label).map(|json| (label, json)))
        .collect::<Result<Vec<_>, _>>()?;
    let filaments = filaments
        .map(|label| read(label).map(|json| (label, json)))
        .collect::<Result<Vec<_>, _>>()?;
    fn preset_files<'a>(files: &'a [(&str, String)]) -> Vec<LoadSettingsFile<'a>> {
        files
            .iter()
            .map(|(label, json)| LoadSettingsFile { label, json })
            .collect()
    }
    ares_core::validate_stl_export_presets(&LoadSettingsRequest {
        settings: &preset_files(&settings),
        filaments: &preset_files(&filaments),
    })
    .map_err(|err| {
        invalid(match err {
            SliceError::InvalidInput(detail) if detail.starts_with("duplicate ") => {
                format!("{detail}\n")
            }
            _ => String::new(),
        })
    })
}
