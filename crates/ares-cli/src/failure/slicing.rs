//! `OrcaSlicer.cpp:7141–7172`: non-strict critical empty-layer refusal.
use super::{CliFailure, FamilyFailure};

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
    let failure = CliFailure::Family(Box::new(FamilyFailure {
        return_code: -100,
        #[cfg(target_os = "linux")]
        plate_index: args.plate.unwrap_or(1),
        #[cfg(target_os = "linux")]
        layer_height: plate.layer_height,
        #[cfg(target_os = "linux")]
        wall_loops: plate.wall_loops,
        #[cfg(target_os = "linux")]
        sparse_infill_density: plate.sparse_infill_density,
        #[cfg(target_os = "linux")]
        error_string: "Failed slicing the model. Please verify the slicing of all plates on Orca Slicer before uploading.".to_owned(),
        upward_machines: upward.to_vec(),
        #[cfg(target_os = "linux")]
        warnings: Vec::new(),
        #[cfg(target_os = "linux")]
        critical_warning: Some(warning.message.clone()),
        stderr: format!(
            "plate {}: found slicing warnings: {}\n",
            args.plate.unwrap_or(1), warning.message,
        ),
        #[cfg(target_os = "linux")]
        output: args.output.clone(),
    }))
    .with_config_substitutions(input_file, crate::substitutions(input));
    #[cfg(target_os = "linux")]
    let failure = failure.with_slicing_warnings(args.plate.unwrap_or(1), plate.warnings.clone());
    Err(failure)
}
