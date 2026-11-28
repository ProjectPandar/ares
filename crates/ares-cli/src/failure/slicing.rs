//! `OrcaSlicer.cpp:7141–7172`: gate export on typed slicing warnings.
#[cfg(target_os = "linux")]
use std::path::Path;

use super::{CliFailure, FamilyFailure};
use ares_core::{PrintWarning, SliceError, SlicedPlate, SlicingNotificationType, WarningLevel};

impl CliFailure {
    pub(crate) fn with_stl_reports(
        self,
        (_plate, _settings): (u32, Option<(f64, i32, f64)>),
        _completed: Vec<(u32, SlicedPlate)>,
    ) -> Self {
        #[cfg(target_os = "linux")]
        {
            let Self::Family(mut family) = self else {
                return self;
            };
            if _plate != 0 {
                family.plate_index = _plate;
            }
            if let Some(settings) = _settings {
                (
                    family.layer_height,
                    family.wall_loops,
                    family.sparse_infill_density,
                ) = settings;
            }
            let warnings = std::mem::take(&mut family.warnings);
            for (id, report) in _completed {
                family
                    .completed
                    .push(crate::result_json::SlicedPlateEntry::from_plate(
                        id, &report,
                    ));
                family
                    .warnings
                    .extend(crate::result_json::slicing_warnings(id, report.warnings));
            }
            family.warnings.extend(warnings);
            Self::Family(family)
        }
        #[cfg(not(target_os = "linux"))]
        self
    }
}

fn is_empty_layer_warning(warning: &PrintWarning) -> bool {
    warning.level == WarningLevel::Critical
        && warning.message_type == SlicingNotificationType::EmptyGcodeLayers
}

pub(crate) fn check(
    args: &crate::SliceArgs,
    plate: &ares_core::SlicedPlate,
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
    let failure = failure.with_upward_report(upward.to_vec());
    #[cfg(target_os = "linux")]
    let failure = failure.with_slicing_warnings(id, plate.warnings.clone());
    Err(failure)
}

pub(crate) fn critical_failure(
    #[cfg(target_os = "linux")] output: &Path,
    id: u32,
    plate: &ares_core::SlicedPlate,
) -> Option<CliFailure> {
    let warning = plate
        .warnings
        .iter()
        .find(|warning| is_empty_layer_warning(warning))?;
    Some(warning_failure(
        #[cfg(target_os = "linux")]
        output,
        id,
        #[cfg(target_os = "linux")]
        plate,
        (warning, true),
    ))
}

pub(crate) fn check_direct(
    (diagnostics, no_check, strict): (&mut crate::diagnostics::DiagnosticLog, bool, bool),
    #[cfg(target_os = "linux")] output: &Path,
    (plate_index, plate): (u32, &SlicedPlate),
    upward: &[String],
) -> Result<(), CliFailure> {
    let mut pending = plate.warnings.as_slice();
    while let Some((warning, remaining)) = pending.split_first() {
        pending = remaining;
        let reject = match warning.level {
            WarningLevel::NonCritical => {
                diagnostics.warning(format_args!(
                    "plate {plate_index}: found NON_CRITICAL slicing warnings: {}",
                    warning.message,
                ));
                strict
            }
            WarningLevel::Critical => {
                diagnostics.warning(format_args!(
                    "plate {plate_index}: found slicing warnings: {}, no_check={}",
                    warning.message,
                    u8::from(no_check),
                ));
                !no_check
                    && matches!(
                        warning.message_type,
                        SlicingNotificationType::EmptyGcodeLayers
                            | SlicingNotificationType::GcodeOverlap
                    )
            }
        };
        if !reject {
            continue;
        }
        let failure = warning_failure(
            #[cfg(target_os = "linux")]
            output,
            plate_index,
            #[cfg(target_os = "linux")]
            plate,
            (warning, false),
        )
        .with_upward_report(upward.to_vec());
        #[cfg(target_os = "linux")]
        let failure = failure.with_slicing_warnings(
            plate_index,
            plate.warnings[..plate.warnings.len() - pending.len()].to_vec(),
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
        downward_machines: Vec::new(),
        #[cfg(target_os = "linux")]
        warnings: Vec::new(),
        #[cfg(target_os = "linux")]
        plate_warning: None,
        #[cfg(target_os = "linux")]
        completed: Vec::new(),
        stderr: String::new(),
        #[cfg(target_os = "linux")]
        output: output.to_owned(),
    })))
}

fn warning_failure(
    #[cfg(target_os = "linux")] output: &Path,
    plate_index: u32,
    #[cfg(target_os = "linux")] plate: &SlicedPlate,
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
        error_string: "Failed slicing the model. Please verify the slicing of all plates on Orca Slicer before uploading.".to_owned(),
        upward_machines: Vec::new(),
        downward_machines: Vec::new(),
        #[cfg(target_os = "linux")]
        warnings: Vec::new(),
        #[cfg(target_os = "linux")]
        plate_warning: Some(warning.message.clone()),
        #[cfg(target_os = "linux")]
        completed: Vec::new(),
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
