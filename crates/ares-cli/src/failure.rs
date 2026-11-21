#[cfg(target_os = "linux")]
use std::path::PathBuf;
use std::{error::Error, io, path::Path};

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
    downward_machines: Vec<String>,
    #[cfg(target_os = "linux")]
    warnings: Vec<crate::result_json::CliWarning>,
    #[cfg(target_os = "linux")]
    critical_warning: Option<String>,
    stderr: String,
    #[cfg(target_os = "linux")]
    output: PathBuf,
}

impl CliFailure {
    /// `CLI_CONFIG_FILE_ERROR`: global configuration admission before plate selection.
    pub(super) fn config_file_error(
        #[cfg(target_os = "linux")] output: &Path,
        stderr: String,
    ) -> Self {
        Self::Family(Box::new(FamilyFailure {
            return_code: -5,
            #[cfg(target_os = "linux")]
            plate_index: 0,
            #[cfg(target_os = "linux")]
            layer_height: 0.0,
            #[cfg(target_os = "linux")]
            wall_loops: 0,
            #[cfg(target_os = "linux")]
            sparse_infill_density: 0.0,
            #[cfg(target_os = "linux")]
            error_string: "The input preset file is invalid and can not be parsed.",
            upward_machines: Vec::new(),
            downward_machines: Vec::new(),
            #[cfg(target_os = "linux")]
            warnings: Vec::new(),
            #[cfg(target_os = "linux")]
            critical_warning: None,
            stderr,
            #[cfg(target_os = "linux")]
            output: output.to_owned(),
        }))
    }

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
            downward_machines: Vec::new(),
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

    pub(super) fn downward_missing(input: &Path, _output: &Path) -> Self {
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
            downward_machines: Vec::new(),
            #[cfg(target_os = "linux")]
            warnings: Vec::new(),
            #[cfg(target_os = "linux")]
            critical_warning: None,
            stderr: format!(
                "operator(): can not find setting file: {}\n",
                input.display()
            ),
            #[cfg(target_os = "linux")]
            output: _output.to_owned(),
        }))
    }

    pub(super) fn downward_config(_output: &Path) -> Self {
        Self::Family(Box::new(FamilyFailure {
            return_code: -5,
            #[cfg(target_os = "linux")]
            plate_index: 0,
            #[cfg(target_os = "linux")]
            layer_height: 0.0,
            #[cfg(target_os = "linux")]
            wall_loops: 0,
            #[cfg(target_os = "linux")]
            sparse_infill_density: 0.0,
            #[cfg(target_os = "linux")]
            error_string: "The input preset file is invalid and can not be parsed.",
            upward_machines: Vec::new(),
            downward_machines: Vec::new(),
            #[cfg(target_os = "linux")]
            warnings: Vec::new(),
            #[cfg(target_os = "linux")]
            critical_warning: None,
            stderr: String::new(),
            #[cfg(target_os = "linux")]
            output: _output.to_owned(),
        }))
    }

    /// `cli_report_config_substitutions` (`OrcaSlicer.cpp:520-534`) after the 3MF load (`:1992`):
    /// the stderr lines precede the family's own, and result.json records each substitution. The
    /// post_process rejection (`:1806`) comes before the report and carries none.
    pub(super) fn with_config_substitutions(
        mut self,
        file: &str,
        substitutions: Vec<ares_core::ConfigSubstitution>,
    ) -> Self {
        if let Self::Family(family) = &mut self
            && family.return_code != -19
        {
            family.stderr = substitution_warnings(file, &substitutions) + &family.stderr;
            #[cfg(target_os = "linux")]
            {
                family.warnings = substitutions
                    .into_iter()
                    .map(
                        |substitution| crate::result_json::CliWarning::ConfigValueSubstituted {
                            file: file.to_owned(),
                            substitution,
                        },
                    )
                    .collect();
            }
        }
        self
    }

    pub(super) fn report(self) -> ! {
        match self {
            Self::Family(failure) => failure.flush_and_exit(),
            Self::Legacy(error) => {
                eprintln!("Error: {error}");
                std::process::exit(1);
            }
        }
    }
}

mod mapping;
mod report;
pub(crate) mod slicing;

pub(super) use mapping::slice_error_to_failure;

/// The stderr lines of `cli_report_config_substitutions` (`OrcaSlicer.cpp:525-527`).
pub(super) fn substitution_warnings(
    file: &str,
    substitutions: &[ares_core::ConfigSubstitution],
) -> String {
    substitutions
        .iter()
        .map(|substitution| {
            format!(
                "warning: {file}: value \"{}\" for {} is not valid, using \"{}\" instead\n",
                substitution.old_value, substitution.key, substitution.new_value
            )
        })
        .collect()
}
