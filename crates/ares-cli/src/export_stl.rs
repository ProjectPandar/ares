use std::{fs, path::Path};

use ares_core::SliceError;

use crate::{CliFailure, SliceArgs, failure::StlExportFailure};

pub(super) fn run(args: &SliceArgs) -> Result<(), CliFailure> {
    let outputdir = args.outputdir.as_deref().unwrap_or_else(|| Path::new("."));
    #[cfg(target_os = "linux")]
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
        (args.selection.is_some(), "--slice"),
        (args.min_save.is_some(), "--min-save"),
        (args.load_slicedata.is_some(), "--load-slicedata"),
        (args.export_slicedata.is_some(), "--export-slicedata"),
        (args.arrange.is_some_and(|value| value != 0), "--arrange"),
        (args.orient != 0, "--orient"),
        (args.plate.is_some(), "--plate"),
        (args.dump_moves.is_some(), "--dump-moves"),
        (args.overrides.is_some(), "--overrides"),
        (args.options.is_some(), "--options"),
        (args.output.is_some(), "-o"),
        (
            args.project_preset_catalog.is_some(),
            "--project-preset-catalog",
        ),
        (args.resources_dir.is_some(), "--resources-dir"),
        (args.load_settings.is_some(), "--load-settings"),
        (args.load_filaments.is_some(), "--load-filaments"),
        (args.progress, "--progress"),
    ];
    if let Some((_, option)) = unsupported.into_iter().find(|(present, _)| *present) {
        return Err(failure(
            StlExportFailure::Unsupported,
            format!("unsupported STL export option {option}"),
        ));
    }
    let bytes = match fs::read(&args.input) {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Err(CliFailure::missing_input(
                &args.input,
                #[cfg(target_os = "linux")]
                &report_path,
            ));
        }
        Err(err) => return Err(failure(StlExportFailure::Input, err.to_string())),
    };
    let filename = args
        .input
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            failure(
                StlExportFailure::Unsupported,
                "invalid source filename".to_owned(),
            )
        })?;
    let artifacts = ares_core::export_stl(&bytes, filename, args.repair.unwrap_or_default())
        .map_err(|err| match err {
            SliceError::UnsupportedProjectFeature(reason) => failure(
                StlExportFailure::Unsupported,
                format!("unsupported {reason}"),
            ),
            _ => failure(StlExportFailure::Input, err.to_string()),
        })?;
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
    Ok(())
}
