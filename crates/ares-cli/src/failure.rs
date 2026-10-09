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
            stderr: format!("No such file: {}\n", input.display()),
            #[cfg(target_os = "linux")]
            output: output.to_owned(),
        }))
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

impl FamilyFailure {
    fn flush_and_exit(self) -> ! {
        #[cfg(target_os = "linux")]
        println!("run found error, return {}, exit...", self.return_code);
        #[cfg(not(target_os = "linux"))]
        println!("run found error, exit");
        eprint!("{}", self.stderr);
        #[cfg(target_os = "linux")]
        let _ = self.record_exit_reason();
        std::process::exit(self.return_code);
    }

    /// `record_exit_reson` (`OrcaSlicer.cpp:416-460`): Linux-only, sorted
    /// keys, tab-indented `dump(1, '\t')` with a trailing newline; failures
    /// to write are swallowed like the upstream `catch (...)`.
    #[cfg(target_os = "linux")]
    fn record_exit_reason(&self) -> Result<(), serde_json::Error> {
        let json_error_string = serde_json::to_string(self.error_string)?;
        let body = format!(
            "{{\n\t\"error_string\": {json_error_string},\n\t\"export_time\": 0,\n\t\
             \"layer_height\": {:?},\n\t\"plate_index\": {},\n\t\"prepare_time\": 0,\n\t\
             \"return_code\": {},\n\t\"sparse_infill_density\": {:?},\n\t\"wall_loops\": {}\n}}\n",
            self.layer_height,
            self.plate_index,
            self.return_code,
            self.sparse_infill_density,
            self.wall_loops,
        );
        let directory = self
            .output
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let _ = fs::write(directory.join("result.json"), body);
        Ok(())
    }
}

impl From<io::Error> for CliFailure {
    fn from(error: io::Error) -> Self {
        Self::Legacy(Box::new(error))
    }
}

impl From<serde_json::Error> for CliFailure {
    fn from(error: serde_json::Error) -> Self {
        Self::Legacy(Box::new(error))
    }
}

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
            stderr,
            #[cfg(target_os = "linux")]
            output: output.to_owned(),
        }))
    };
    match error {
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
        // OrcaSlicer.cpp:5681-5690 selects the active plate; :5723 passes
        // index+1 to record_exit_reson, and :433 writes that plate_id.
        // The progress callback's m_plate_index (:218,:232) is separate.
        SliceError::NoSuitableObjects(_) => family(
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
            "One of the plate is empty or has no object fully inside it. Please check that the 3mf contains no empty plate in Orca Slicer before uploading.",
            String::new(),
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
            format!("{message}\n"),
        ),
        SliceError::CustomGCode {
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
