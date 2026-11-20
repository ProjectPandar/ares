//! `OrcaSlicer.cpp:7141–7172`: gate export on typed slicing warnings.
use super::{CliFailure, FamilyFailure};
use ares_core::{PrintWarning, SliceError, SlicedPlate, SlicingNotificationType, WarningLevel};

fn is_empty_layer_warning(warning: &PrintWarning) -> bool {
    warning.level == WarningLevel::Critical
        && warning.message_type == SlicingNotificationType::EmptyGcodeLayers
}

fn rejects_checked_slice(warning: &PrintWarning) -> bool {
    warning.level == WarningLevel::Critical
        && matches!(
            warning.message_type,
            SlicingNotificationType::EmptyGcodeLayers | SlicingNotificationType::GcodeOverlap
        )
}

pub(crate) fn check(
    args: &crate::SliceArgs,
    plate: &ares_core::SlicedPlate,
    input_file: &str,
    input: &[u8],
    upward: &[String],
) -> Result<(), CliFailure> {
    let Some(warning) = plate
        .warnings
        .iter()
        .find(|warning| is_empty_layer_warning(warning))
    else {
        return Ok(());
    };
    let plate_index = args.plate.unwrap_or(1);
    let failure = warning_failure(
        #[cfg(target_os = "linux")]
        &args.output,
        plate_index,
        #[cfg(target_os = "linux")]
        plate,
        upward,
        (warning, true),
    )
    .with_config_substitutions(input_file, crate::substitutions(input));
    #[cfg(target_os = "linux")]
    let failure = failure.with_slicing_warnings(plate_index, plate.warnings.clone());
    Err(failure)
}

pub(crate) fn check_direct(
    (diagnostics, no_check): (&mut crate::diagnostics::DiagnosticLog, bool),
    #[cfg(target_os = "linux")] output: &std::path::Path,
    (plate_index, plate): (u32, &SlicedPlate),
    upward: &[String],
    project: Option<(&str, &[u8])>,
) -> Result<(), CliFailure> {
    for warning in plate
        .warnings
        .iter()
        .filter(|warning| warning.level == WarningLevel::Critical)
    {
        diagnostics.warning(format_args!(
            "plate {plate_index}: found slicing warnings: {}, no_check={}",
            warning.message,
            u8::from(no_check),
        ));
        if no_check || !rejects_checked_slice(warning) {
            continue;
        }
        let mut failure = warning_failure(
            #[cfg(target_os = "linux")]
            output,
            plate_index,
            #[cfg(target_os = "linux")]
            plate,
            upward,
            (warning, false),
        );
        if let Some((file, bytes)) = project {
            failure = failure.with_config_substitutions(file, crate::substitutions(bytes));
        }
        #[cfg(target_os = "linux")]
        let failure = failure.with_slicing_warnings(
            plate_index,
            plate
                .warnings
                .iter()
                .take_while(|warning| !rejects_checked_slice(warning))
                .cloned()
                .collect(),
        );
        return Err(failure);
    }
    Ok(())
}

pub(crate) fn path_conflict(
    error: &SliceError,
    diagnostics: &mut crate::diagnostics::DiagnosticLog,
    #[cfg(target_os = "linux")] output: &std::path::Path,
    plate_index: u32,
    #[cfg(target_os = "linux")] plate: &SlicedPlate,
) -> Option<CliFailure> {
    let SliceError::GcodePathConflict([first, second]) = error else {
        return None;
    };
    // Print.cpp:3963 and OrcaSlicer.cpp:7136 write to the error log, not stderr.
    diagnostics.error(format_args!(
        "gcode path conflicts found between {first} and {second}"
    ));
    diagnostics.error(format_args!(
        "plate {plate_index}: found slicing result conflict!"
    ));
    Some(CliFailure::Family(Box::new(FamilyFailure {
        return_code: -101,
        #[cfg(target_os = "linux")]
        plate_index,
        #[cfg(target_os = "linux")]
        layer_height: plate.layer_height,
        #[cfg(target_os = "linux")]
        wall_loops: plate.wall_loops,
        #[cfg(target_os = "linux")]
        sparse_infill_density: plate.sparse_infill_density,
        #[cfg(target_os = "linux")]
        error_string: " G-code conflicts detected after slicing. Please make sure the 3mf file can be successfully sliced in the latest Orca Slicer. If the file slices normally in Orca Slicer, try moving the wipe tower further from other models, as we use more conservative parameters for it during upload.".into(),
        upward_machines: Vec::new(),
        #[cfg(target_os = "linux")]
        warnings: Vec::new(),
        #[cfg(target_os = "linux")]
        plate_warning: None,
        stderr: String::new(),
        #[cfg(target_os = "linux")]
        output: output.to_owned(),
    })))
}

pub(crate) fn check_strict(
    #[cfg(target_os = "linux")] output: &std::path::Path,
    plate_index: u32,
    plate: &SlicedPlate,
    upward: &[String],
    project: Option<(&str, &[u8])>,
) -> Result<(), CliFailure> {
    let Some(warning) = plate
        .warnings
        .iter()
        .find(|warning| warning.level == WarningLevel::NonCritical)
    else {
        return Ok(());
    };
    let mut failure = warning_failure(
        #[cfg(target_os = "linux")]
        output,
        plate_index,
        #[cfg(target_os = "linux")]
        plate,
        upward,
        (warning, false),
    );
    if let Some((file, bytes)) = project {
        failure = failure.with_config_substitutions(file, crate::substitutions(bytes));
    }
    #[cfg(target_os = "linux")]
    let failure = failure.with_slicing_warnings(plate_index, vec![warning.clone()]);
    Err(failure)
}

fn warning_failure(
    #[cfg(target_os = "linux")] output: &std::path::Path,
    plate_index: u32,
    #[cfg(target_os = "linux")] plate: &SlicedPlate,
    upward: &[String],
    (warning, emit_stderr): (&PrintWarning, bool),
) -> CliFailure {
    CliFailure::Family(Box::new(FamilyFailure {
        return_code: -100,
        #[cfg(target_os = "linux")]
        plate_index,
        #[cfg(target_os = "linux")]
        layer_height: plate.layer_height,
        #[cfg(target_os = "linux")]
        wall_loops: plate.wall_loops,
        #[cfg(target_os = "linux")]
        sparse_infill_density: plate.sparse_infill_density,
        #[cfg(target_os = "linux")]
        error_string: "Failed slicing the model. Please verify the slicing of all plates on Orca Slicer before uploading.".into(),
        upward_machines: upward.to_vec(),
        #[cfg(target_os = "linux")]
        warnings: Vec::new(),
        #[cfg(target_os = "linux")]
        plate_warning: Some(warning.message.clone()),
        stderr: if emit_stderr {
            format!(
                "plate {plate_index}: found slicing warnings: {}\n",
                warning.message
            )
        } else {
            String::new()
        },
        #[cfg(target_os = "linux")]
        output: output.to_owned(),
    }))
}
