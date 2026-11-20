//! `OrcaSlicer.cpp:7141–7172`: gate export on typed slicing warnings.
use super::{CliFailure, FamilyFailure};
use ares_core::{PrintWarning, SlicedPlate, WarningLevel};

pub(crate) fn check(
    args: &crate::SliceArgs,
    plate: &ares_core::SlicedPlate,
    input_file: &str,
    input: &[u8],
    upward: &[String],
) -> Result<(), CliFailure> {
    let Some(warning) = plate.warnings.iter().find(|warning| {
        warning.level == ares_core::WarningLevel::Critical
            && warning.message_type == ares_core::SlicingNotificationType::EmptyGcodeLayers
    }) else {
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
        warning,
    )
    .with_config_substitutions(input_file, crate::substitutions(input));
    #[cfg(target_os = "linux")]
    let failure = failure.with_slicing_warnings(plate_index, plate.warnings.clone());
    Err(failure)
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
        warning,
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
    warning: &PrintWarning,
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
        error_string: "Failed slicing the model. Please verify the slicing of all plates on Orca Slicer before uploading.",
        upward_machines: upward.to_vec(),
        #[cfg(target_os = "linux")]
        warnings: Vec::new(),
        #[cfg(target_os = "linux")]
        plate_warning: Some(warning.message.clone()),
        stderr: if warning.level == WarningLevel::Critical {
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
