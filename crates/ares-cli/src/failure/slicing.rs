//! `OrcaSlicer.cpp:7141–7172`: non-strict critical empty-layer refusal.
#[cfg(target_os = "linux")]
use std::path::Path;

use super::{CliFailure, FamilyFailure};

pub(crate) fn check(
    args: &crate::SliceArgs,
    plate: &ares_core::SlicedPlate,
    input_file: &str,
    input: &[u8],
    upward: &[String],
) -> Result<(), CliFailure> {
    let id = args.plate.unwrap_or(1);
    let Some(failure) = critical_failure(
        #[cfg(target_os = "linux")]
        args.output(),
        id,
        plate,
    ) else {
        return Ok(());
    };
    let failure = failure
        .with_upward_report(upward.to_vec())
        .with_config_substitutions(input_file, crate::substitutions(input));
    #[cfg(target_os = "linux")]
    let failure = failure.with_slicing_warnings(id, plate.warnings.clone());
    Err(failure)
}

pub(crate) fn critical_failure(
    #[cfg(target_os = "linux")] output: &Path,
    id: u32,
    plate: &ares_core::SlicedPlate,
) -> Option<CliFailure> {
    let warning = plate.warnings.iter().find(|warning| {
        warning.level == ares_core::WarningLevel::Critical
            && warning.message_type == ares_core::SlicingNotificationType::EmptyGcodeLayers
    })?;
    Some(CliFailure::Family(Box::new(FamilyFailure {
        return_code: -100,
        #[cfg(target_os = "linux")]
        plate_index: id,
        #[cfg(target_os = "linux")]
        layer_height: plate.layer_height,
        #[cfg(target_os = "linux")]
        wall_loops: plate.wall_loops,
        #[cfg(target_os = "linux")]
        sparse_infill_density: plate.sparse_infill_density,
        #[cfg(target_os = "linux")]
        error_string: "Failed slicing the model. Please verify the slicing of all plates on Orca Slicer before uploading.",
        upward_machines: Vec::new(),
        #[cfg(target_os = "linux")]
        warnings: Vec::new(),
        #[cfg(target_os = "linux")]
        critical_warning: Some(warning.message.clone()),
        #[cfg(target_os = "linux")]
        completed: Vec::new(),
        stderr: format!("plate {id}: found slicing warnings: {}\n", warning.message),
        #[cfg(target_os = "linux")]
        output: output.to_owned(),
    })))
}
