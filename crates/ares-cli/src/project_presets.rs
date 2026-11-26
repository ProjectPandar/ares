use std::{fs, io, path::Path};

use crate::{
    SliceArgs,
    failure::{CliFailure, slice_error_to_failure},
};

pub(super) fn load_resources_catalog(
    resources_dir: &Path,
) -> Result<ares_core::PresetCatalogHandle, CliFailure> {
    let profiles_dir = resources_dir.join("profiles");
    let entries = match fs::read_dir(&profiles_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(ares_core::PresetCatalogHandle::new());
        }
        Err(error) => return Err(error.into()),
    };
    let mut vendor_paths = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    vendor_paths.sort();
    let mut catalog = ares_core::PresetCatalogHandle::new();
    for index_path in vendor_paths {
        let id = index_path
            .file_stem()
            .and_then(|name| name.to_str())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid vendor filename"))?;
        let index_bytes = fs::read(&index_path)?;
        let Some(manifest) = ares_core::VendorManifest::parse_optional(id, &index_bytes)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?
        else {
            continue;
        };
        let files = crate::preset_sources::read_vendor_files(&profiles_dir, &manifest)?;
        catalog
            .add_vendor_manifest(
                manifest,
                files
                    .iter()
                    .map(|(sub_path, bytes)| (sub_path.as_str(), bytes.as_slice())),
            )
            .map_err(|error| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("invalid vendor {id} profile JSON: {error}"),
                )
            })?;
    }
    Ok(catalog)
}

pub(super) fn retained_printer_catalog(
    resources: &Path,
    source: Option<&ares_core::UpwardMachineSource>,
    explicit: bool,
) -> Result<Option<ares_core::PresetCatalogHandle>, CliFailure> {
    if explicit || (source.is_some() && resources.join("profiles").try_exists()?) {
        return load_resources_catalog(resources).map(Some);
    }
    Ok(None)
}

pub(super) enum Entry<'a> {
    Catalog(&'a Path),
    Native(&'a ares_core::NativeRefreshPresets),
}

pub(super) fn slice(
    args: &SliceArgs,
    entry: Entry<'_>,
    mut admitted: ares_core::CliPresetProject<'_>,
    overrides: &ares_core::ConfigOverrides,
    context: ares_core::SlicerContext<'_>,
) -> Result<(), CliFailure> {
    let filename = args
        .input()
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid model filename"))?;
    let selection = if args.uptodate {
        r#"{"from_project":true,"uptodate":true}"#
    } else {
        r#"{"from_project":true,"uptodate":false}"#
    };

    let metadata = ares_core::GenerationMetadata::default();
    let input_file = args.input().to_string_lossy();
    let substitutions = admitted.substitutions().to_vec();
    let source = admitted
        .take_upward_source()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?;
    let mut upward_report = Vec::new();
    let mut sliced_plate = ares_core::SlicedPlate::default();
    let upward = ares_core::CliUpwardMachine {
        source,
        input: match entry {
            Entry::Catalog(_) => ares_core::UpwardMachineInput::Embedded,
            Entry::Native(refresh) => ares_core::UpwardMachineInput::RetainedRefresh(refresh),
        },
        report: &mut upward_report,
    };
    enum SliceOutcome {
        Buffered(ares_core::ProcessedGCode),
        Streamed(
            ares_core::StreamedGCode,
            crate::gcode_file_sink::AtomicFileSink,
        ),
    }
    let result = match entry {
        Entry::Catalog(path) => {
            let catalog = fs::read_to_string(path).map_err(|error| {
                io::Error::new(
                    error.kind(),
                    format!(
                        "cannot read project preset catalog {}: {error}",
                        path.display()
                    ),
                )
            })?;
            admitted
                .slice_with_profiles(
                    ares_core::CliProfileSliceRequest {
                        orient: args.orient == Some(1),
                        profile: ares_core::ProfileSliceRequest {
                            filename,
                            catalog_json: &catalog,
                            selection_json: selection,
                            plate: args.plate,
                            overrides,
                            placement: ares_core::Placement::None,
                        },
                        record_moves: args.dump_moves.is_some(),
                        sliced_plate: &mut sliced_plate,
                        upward,
                    },
                    metadata,
                    context,
                )
                .map(SliceOutcome::Buffered)
        }
        Entry::Native(refresh) => {
            let mut sink = crate::gcode_file_sink::AtomicFileSink::new(args.output().to_owned());
            let streamed = admitted.slice_project_to_sink(
                ares_core::CliProjectSliceRequest {
                    orient: args.orient == Some(1),
                    plate: args.plate,
                    overrides,
                    record_moves: args.dump_moves.is_some(),
                    sliced_plate: &mut sliced_plate,
                    upward,
                },
                (metadata, (None, Some(refresh))),
                context,
                &mut sink,
            );
            if streamed.is_err()
                && let Some(io_error) = sink.take_error()
            {
                return Err(io_error.into());
            }
            streamed.map(|streamed| SliceOutcome::Streamed(streamed, sink))
        }
    };
    let outcome = match result {
        Ok(outcome) => outcome,
        Err(error) => {
            crate::failure::slicing::check(
                args,
                &sliced_plate,
                &input_file,
                &substitutions,
                &upward_report,
            )?;
            let failure = slice_error_to_failure(
                error,
                args.output(),
                #[cfg(target_os = "linux")]
                args.plate.unwrap_or(1),
            )
            .with_upward_report(upward_report)
            .with_config_substitutions(&input_file, substitutions);
            #[cfg(target_os = "linux")]
            let failure =
                failure.with_slicing_warnings(args.plate.unwrap_or(1), sliced_plate.warnings);
            return Err(failure);
        }
    };
    crate::failure::slicing::check(
        args,
        &sliced_plate,
        &input_file,
        &substitutions,
        &upward_report,
    )?;
    match outcome {
        SliceOutcome::Buffered(processed) => {
            if let Some(path) = &args.dump_moves {
                fs::write(path, crate::moves_csv::format(&processed.result))?;
            }
            fs::write(args.output(), processed.gcode)?;
        }
        SliceOutcome::Streamed(streamed, sink) => {
            if let Some(path) = &args.dump_moves {
                fs::write(path, crate::moves_csv::format(&streamed.result))?;
            }
            sink.commit()?;
        }
    }
    crate::report_success(
        &input_file,
        substitutions,
        #[cfg(target_os = "linux")]
        crate::result_json::Success {
            strict_mode: false,
            output: args.output(),
            plate: args.plate.unwrap_or(1),
            plate_index: args.plate.unwrap_or(0),
            sliced_plate,
            upward_machines: &upward_report,
            downward_machines: &[],
        },
    );
    Ok(())
}

pub(super) fn slice_embedded(
    args: &SliceArgs,
    mut admitted: ares_core::CliPresetProject<'_>,
    resources: &Path,
    overrides: &ares_core::ConfigOverrides,
    context: ares_core::SlicerContext<'_>,
) -> Result<(), CliFailure> {
    let metadata = ares_core::GenerationMetadata::default();
    let input_file = args.input().to_string_lossy();
    let substitutions = admitted.substitutions().to_vec();
    let source = admitted
        .take_upward_source()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?;
    let machine_full = crate::compatible_machine::read_machine_full(source.as_ref(), resources)?;
    let system_catalog =
        retained_printer_catalog(resources, source.as_ref(), args.resources_dir.is_some())?;
    let mut upward_report = Vec::new();
    let mut sliced_plate =
        ares_core::SlicedPlate::with_size(args.project_actions.has_downward_targets());
    let upward = ares_core::CliUpwardMachine {
        source,
        input: ares_core::UpwardMachineInput::FetchOriginal(machine_full.as_deref()),
        report: &mut upward_report,
    };
    // OrcaSlicer.cpp:5681-5690 iterates from plate index0 when
    // --slice0 is used, or selects the explicit one-based plate.
    // This byte API slices one view; its default is that first plate.
    let request = ares_core::CliProjectSliceRequest {
        orient: args.orient == Some(1),
        plate: args.plate,
        overrides,
        record_moves: args.dump_moves.is_some(),
        upward,
        sliced_plate: &mut sliced_plate,
    };
    let mut sink = crate::gcode_file_sink::AtomicFileSink::new(args.output().to_owned());
    let streamed = match admitted.slice_project_to_sink(
        request,
        (metadata, (system_catalog.as_ref(), None)),
        context,
        &mut sink,
    ) {
        Ok(streamed) => streamed,
        Err(error) => {
            if let Some(io_error) = sink.take_error() {
                return Err(io_error.into());
            }
            crate::failure::slicing::check(
                args,
                &sliced_plate,
                &input_file,
                &substitutions,
                &upward_report,
            )?;
            let failure = slice_error_to_failure(
                error,
                args.output(),
                #[cfg(target_os = "linux")]
                args.plate.unwrap_or(1),
            )
            .with_upward_report(upward_report)
            .with_config_substitutions(&input_file, substitutions);
            #[cfg(target_os = "linux")]
            let failure =
                failure.with_slicing_warnings(args.plate.unwrap_or(1), sliced_plate.warnings);
            return Err(failure);
        }
    };
    if let Some(path) = &args.dump_moves {
        crate::failure::slicing::check(
            args,
            &sliced_plate,
            &input_file,
            &substitutions,
            &upward_report,
        )?;
        fs::write(path, crate::moves_csv::format(&streamed.result))?;
    }
    let _downward = crate::downward::report(
        args,
        &sliced_plate,
        (&input_file, &substitutions),
        &upward_report,
    )?;
    sink.commit()?;
    crate::report_success(
        &input_file,
        substitutions,
        #[cfg(target_os = "linux")]
        crate::result_json::Success {
            strict_mode: false,
            output: args.output(),
            plate: args.plate.unwrap_or(1),
            plate_index: args.plate.unwrap_or(0),
            sliced_plate,
            upward_machines: &upward_report,
            downward_machines: &_downward,
        },
    );
    Ok(())
}
