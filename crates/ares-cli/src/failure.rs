#[cfg(target_os = "linux")]
use std::path::PathBuf;
use std::{error::Error, fs, io, path::Path};

use ares_core::SliceError;

/// CLI failure reporting mirrors the upstream adapter: family failures run
/// `flush_and_exit` (`OrcaSlicer.cpp:396-411` — the stdout line and the
/// negative return, which becomes the POSIX exit 256+n) plus
/// `record_exit_reson` (`:416-460`, Linux-only result.json); every other
/// error keeps Rust's original `Error: <Debug>` + exit 1 shape.
pub(super) enum CliFailure {
    Family(Box<FamilyFailure>),
    Legacy(Box<dyn Error>),
}

// Upstream result metadata is only consumed on Linux (OrcaSlicer.cpp:416-460).
pub(super) struct FamilyFailure {
    return_code: i32,
    #[cfg(target_os = "linux")]
    plate_index: u32,
    #[cfg(target_os = "linux")]
    layer_height: f64,
    #[cfg(target_os = "linux")]
    wall_loops: i32,
    #[cfg(target_os = "linux")]
    sparse_infill_density: f64,
    #[cfg(target_os = "linux")]
    error_string: &'static str,
    upward_machines: Vec<String>,
    #[cfg(target_os = "linux")]
    warnings: Vec<crate::result_json::CliWarning>,
    #[cfg(target_os = "linux")]
    critical_warning: Option<String>,
    stderr: String,
    #[cfg(target_os = "linux")]
    output: PathBuf,
}

impl CliFailure {
    pub(super) fn missing_input(input: &Path, #[cfg(target_os = "linux")] output: &Path) -> Self {
        Self::Family(Box::new(FamilyFailure {
            return_code: -3,
            #[cfg(target_os = "linux")]
            plate_index: 0,
            #[cfg(target_os = "linux")]
            layer_height: 0.0,
            #[cfg(target_os = "linux")]
            wall_loops: 0,
            #[cfg(target_os = "linux")]
            sparse_infill_density: 0.0,
            #[cfg(target_os = "linux")]
            error_string: "The input files to the slicer are not found.",
            upward_machines: Vec::new(),
            #[cfg(target_os = "linux")]
            warnings: Vec::new(),
            #[cfg(target_os = "linux")]
            critical_warning: None,
            stderr: format!("No such file: {}\n", input.display()),
            #[cfg(target_os = "linux")]
            output: output.to_owned(),
        }))
    }

    pub(super) fn with_upward_report(mut self, upward: Vec<String>) -> Self {
        if let Self::Family(family) = &mut self {
            family.upward_machines = upward;
        }
        self
    }

    pub(super) fn report(self) -> ! {
        match self {
            Self::Family(failure) => failure.flush_and_exit(),
            Self::Legacy(error) => {
                eprintln!("Error: {error:?}");
                std::process::exit(1);
            }
        }
    }
}

mod report;
pub(crate) mod slicing;

pub(super) use report::substitution_warnings;

/// Exhaustive variant → CLI code mapping (`Utils.hpp:24-59`). The four
/// family variants carry the result.json payload; everything else keeps
/// the legacy `Error:` channel — those surfaces have no native family
/// emulation in this slice (recorded in clusters.md).
pub(super) fn slice_error_to_failure(
    error: SliceError,
    output: &Path,
    #[cfg(target_os = "linux")] selected_plate_index: u32,
) -> CliFailure {
    let family = |return_code,
                  #[cfg(target_os = "linux")] plate_index,
                  #[cfg(target_os = "linux")] layer_height,
                  #[cfg(target_os = "linux")] wall_loops,
                  #[cfg(target_os = "linux")] sparse,
                  #[cfg(target_os = "linux")] error_string,
                  stderr| {
        CliFailure::Family(Box::new(FamilyFailure {
            return_code,
            #[cfg(target_os = "linux")]
            plate_index,
            #[cfg(target_os = "linux")]
            layer_height,
            #[cfg(target_os = "linux")]
            wall_loops,
            #[cfg(target_os = "linux")]
            sparse_infill_density: sparse,
            #[cfg(target_os = "linux")]
            error_string,
            upward_machines: Vec::new(),
            #[cfg(target_os = "linux")]
            warnings: Vec::new(),
            #[cfg(target_os = "linux")]
            critical_warning: None,
            stderr,
            #[cfg(target_os = "linux")]
            output: output.to_owned(),
        }))
    };
    match error {
        SliceError::InvalidPrinterTechnology => family(
            -7,
            #[cfg(target_os = "linux")]
            0,
            #[cfg(target_os = "linux")]
            0.0,
            #[cfg(target_os = "linux")]
            0,
            #[cfg(target_os = "linux")]
            0.0,
            #[cfg(target_os = "linux")]
            "Unsupported printer technology (not FDM).",
            "invalid printer_technology \n".into(),
        ),
        SliceError::PostProcessNotSupported => family(
            -19,
            #[cfg(target_os = "linux")]
            0,
            #[cfg(target_os = "linux")]
            0.0,
            #[cfg(target_os = "linux")]
            0,
            #[cfg(target_os = "linux")]
            0.0,
            #[cfg(target_os = "linux")]
            "post_process is not supported under CLI.",
            String::new(),
        ),
        SliceError::ProcessNotCompatible => family(
            -17,
            #[cfg(target_os = "linux")]
            0,
            #[cfg(target_os = "linux")]
            0.0,
            #[cfg(target_os = "linux")]
            0,
            #[cfg(target_os = "linux")]
            0.0,
            #[cfg(target_os = "linux")]
            "The selected printer is not compatible with the process preset in the 3mf.",
            String::new(),
        ),
        // OrcaSlicer.cpp:6522-6531 selects the active plate; :6572 passes
        // index+1 to record_exit_reson, and :552 writes that plate_id.
        // The progress callback's m_plate_index (:305,:319) is separate.
        // The reason is cli_errors[CLI_NO_SUITABLE_OBJECTS] (:220).
        SliceError::InvalidCliScale(_)
        | SliceError::InvalidCliGroundFace(_)
        | SliceError::CliArrangeFailed => {
            let invalid_params = !matches!(error, SliceError::CliArrangeFailed);
            family(
                if invalid_params { -2 } else { -21 },
                #[cfg(target_os = "linux")]
                0,
                #[cfg(target_os = "linux")]
                0.0,
                #[cfg(target_os = "linux")]
                0,
                #[cfg(target_os = "linux")]
                0.0,
                #[cfg(target_os = "linux")]
                if invalid_params {
                    "Invalid parameters to the slicer."
                } else {
                    "An error occurred when auto-arranging object(s)."
                },
                format!("{error}\n"),
            )
        }
        SliceError::NoSuitableObjects(_) | SliceError::NoSuitableObjectsAfterCliArrange => family(
            -50,
            #[cfg(target_os = "linux")]
            selected_plate_index,
            #[cfg(target_os = "linux")]
            0.0,
            #[cfg(target_os = "linux")]
            0,
            #[cfg(target_os = "linux")]
            0.0,
            #[cfg(target_os = "linux")]
            if matches!(error, SliceError::NoSuitableObjectsAfterCliArrange) {
                ares_core::CliProjectTransforms::NO_OBJECTS_AFTER_ARRANGE
            } else {
                "The plate has nothing to slice: it is empty, or no object on it is both printable and fully inside the print volume."
            },
            if matches!(error, SliceError::NoSuitableObjectsAfterCliArrange) {
                format!("{error}\n")
            } else {
                String::new()
            },
        ),
        SliceError::InvalidValuesInProject(detail) => family(
            -18,
            #[cfg(target_os = "linux")]
            0,
            #[cfg(target_os = "linux")]
            0.0,
            #[cfg(target_os = "linux")]
            0,
            #[cfg(target_os = "linux")]
            0.0,
            #[cfg(target_os = "linux")]
            "Invalid parameter value(s) included in the 3mf file.",
            format!("Param values in 3mf/config error: \n{detail}\n"),
        ),
        SliceError::Validate {
            message,
            #[cfg(target_os = "linux")]
            plate_index,
            #[cfg(target_os = "linux")]
            layer_height,
            #[cfg(target_os = "linux")]
            wall_loops,
            #[cfg(target_os = "linux")]
            sparse_infill_density,
            ..
        } => family(
            -51,
            #[cfg(target_os = "linux")]
            plate_index,
            #[cfg(target_os = "linux")]
            layer_height,
            #[cfg(target_os = "linux")]
            wall_loops,
            #[cfg(target_os = "linux")]
            sparse_infill_density,
            #[cfg(target_os = "linux")]
            "There are some incorrect slicing parameters in the 3mf. Please verify the slicing of all plates in Orca Slicer before uploading.",
            format!("{message}\n"),
        ),
        SliceError::SequentialPrintCollision {
            message,
            #[cfg(target_os = "linux")]
            plate_index,
            #[cfg(target_os = "linux")]
            layer_height,
            #[cfg(target_os = "linux")]
            wall_loops,
            #[cfg(target_os = "linux")]
            sparse_infill_density,
            ..
        } => family(
            -63,
            #[cfg(target_os = "linux")]
            plate_index,
            #[cfg(target_os = "linux")]
            layer_height,
            #[cfg(target_os = "linux")]
            wall_loops,
            #[cfg(target_os = "linux")]
            sparse_infill_density,
            #[cfg(target_os = "linux")]
            "Object conflicts were detected when using print-by-object mode. Please verify the slicing of all plates in Orca Slicer before uploading.",
            format!("{message}\n"),
        ),
        SliceError::LayeredPrintCollision {
            message,
            #[cfg(target_os = "linux")]
            plate_index,
            #[cfg(target_os = "linux")]
            layer_height,
            #[cfg(target_os = "linux")]
            wall_loops,
            #[cfg(target_os = "linux")]
            sparse_infill_density,
            ..
        } => family(
            -64,
            #[cfg(target_os = "linux")]
            plate_index,
            #[cfg(target_os = "linux")]
            layer_height,
            #[cfg(target_os = "linux")]
            wall_loops,
            #[cfg(target_os = "linux")]
            sparse_infill_density,
            #[cfg(target_os = "linux")]
            "Object conflicts were detected. Please verify the slicing of all plates in Orca Slicer before uploading.",
            // Print.cpp:1734/5634 stores this validation string with a trailing newline;
            // OrcaSlicer.cpp:7015 writes it through cerr and appends std::endl.
            format!("{message}\n\n"),
        ),
        // OrcaSlicer.cpp:6286-6291: generic slicing/export exceptions share -100.
        SliceError::Slicing {
            message,
            #[cfg(target_os = "linux")]
            plate_index,
            #[cfg(target_os = "linux")]
            layer_height,
            #[cfg(target_os = "linux")]
            wall_loops,
            #[cfg(target_os = "linux")]
            sparse_infill_density,
            ..
        }
        | SliceError::CustomGCode {
            message,
            #[cfg(target_os = "linux")]
            plate_index,
            #[cfg(target_os = "linux")]
            layer_height,
            #[cfg(target_os = "linux")]
            wall_loops,
            #[cfg(target_os = "linux")]
            sparse_infill_density,
            ..
        } => family(
            -100,
            #[cfg(target_os = "linux")]
            plate_index,
            #[cfg(target_os = "linux")]
            layer_height,
            #[cfg(target_os = "linux")]
            wall_loops,
            #[cfg(target_os = "linux")]
            sparse_infill_density,
            #[cfg(target_os = "linux")]
            "Failed slicing the model. Please verify the slicing of all plates on Orca Slicer before uploading.",
            format!("{message}\n"),
        ),
        SliceError::ToolpathOutsidePrintableArea {
            gcode,
            #[cfg(target_os = "linux")]
            plate_index,
            #[cfg(target_os = "linux")]
            layer_height,
            #[cfg(target_os = "linux")]
            wall_loops,
            #[cfg(target_os = "linux")]
            sparse_infill_density,
            ..
        } => {
            if let Err(error) = fs::write(output, gcode) {
                return error.into();
            }
            family(
                -102,
                #[cfg(target_os = "linux")]
                plate_index,
                #[cfg(target_os = "linux")]
                layer_height,
                #[cfg(target_os = "linux")]
                wall_loops,
                #[cfg(target_os = "linux")]
                sparse_infill_density,
                #[cfg(target_os = "linux")]
                "Found G-code in unprintable area of multi-extruder printers after slicing. Please make sure the 3mf file can be successfully sliced in the latest Orca Slicer.",
                String::new(),
            )
        }
        SliceError::EmptyInput
        | SliceError::InvalidInput(_)
        | SliceError::ProjectSlicingIncomplete
        | SliceError::UnsupportedProjectFeature(_) => CliFailure::Legacy(Box::new(error)),
    }
}
