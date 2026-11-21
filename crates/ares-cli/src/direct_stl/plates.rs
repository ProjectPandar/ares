use std::path::Path;

use ares_core::{
    LoadSettingsSliceRequest, Model, SliceError, SlicedPlate, SlicedStlPlate, SlicerContext,
};

use crate::{
    diagnostics::DiagnosticLog,
    failure::{CliFailure, slice_error_to_failure, slicing},
};

pub(super) fn slice_arranged(
    (input, output): (&Path, &Path),
    (diagnostics, no_check, strict, statistics): (&mut DiagnosticLog, bool, bool, bool),
    (models, request, selection, context): (
        &[(&str, Model)],
        LoadSettingsSliceRequest<'_>,
        u32,
        SlicerContext<'_>,
    ),
    completed: &mut Vec<(u32, SlicedPlate)>,
) -> Result<(), CliFailure> {
    let failure = |failure: ares_core::StlPlateFailure,
                   diagnostics: &mut DiagnosticLog,
                   completed: &mut Vec<(u32, SlicedPlate)>| {
        slice_failure(
            (input, output),
            diagnostics,
            *failure.error,
            (failure.plate, failure.report.as_deref()),
        )
        .with_stl_reports(
            (failure.plate, failure.print_settings),
            std::mem::take(completed),
        )
    };
    let plates = ares_core::slice_stl_models_with_load_settings(
        models, request, selection, context, statistics,
    )
    .map_err(|error| failure(error, diagnostics, completed))?;
    for plate in plates {
        let plate = plate.map_err(|error| failure(error, diagnostics, completed))?;
        let id = plate.id;
        let entry = publish(output, (diagnostics, no_check, strict), plate, statistics)
            .map_err(|error| error.with_stl_reports((id, None), std::mem::take(completed)))?;
        completed.push(entry);
    }
    Ok(())
}

pub(super) fn slice_failure(
    (input, output): (&Path, &Path),
    diagnostics: &mut DiagnosticLog,
    error: SliceError,
    (plate, report): (u32, Option<&SlicedPlate>),
) -> CliFailure {
    if let Some(_report) = report
        && let Some(failure) = slicing::path_conflict(
            &error,
            diagnostics,
            #[cfg(target_os = "linux")]
            output,
            plate,
            #[cfg(target_os = "linux")]
            _report,
        )
    {
        return failure;
    }
    if let SliceError::TriangleCountExceedsLimit { count, limit } = &error {
        // OrcaSlicer.cpp:6639 writes to the file sink, not to stderr.
        diagnostics.error(format_args!(
            "plate {plate}: triangle count {count} exceeds the limit:{limit}"
        ));
    }
    match error {
        SliceError::OptionStlModelRead(_) => CliFailure::invalid_stl_input(
            input,
            #[cfg(target_os = "linux")]
            output,
        ),
        error => slice_error_to_failure(
            error,
            output,
            #[cfg(target_os = "linux")]
            plate,
        ),
    }
}

pub(super) fn publish(
    output: &Path,
    options: (&mut DiagnosticLog, bool, bool),
    SlicedStlPlate {
        id,
        processed,
        report,
    }: SlicedStlPlate,
    statistics: bool,
) -> Result<(u32, SlicedPlate), CliFailure> {
    slicing::check_direct(
        options,
        #[cfg(target_os = "linux")]
        output,
        (id, &report),
        &[],
        None,
    )?;
    crate::statistics::write(
        &output.with_file_name(format!("plate_{id}.gcode")),
        processed,
        statistics,
        id,
    )?;
    Ok((id, report))
}
