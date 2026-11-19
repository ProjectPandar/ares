use super::*;

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

#[cfg(target_os = "linux")]
impl CliFailure {
    pub(crate) fn with_slicing_warnings(
        mut self,
        plate: u32,
        warnings: Vec<ares_core::PrintWarning>,
    ) -> Self {
        if let Self::Family(family) = &mut self {
            family
                .warnings
                .extend(crate::result_json::slicing_warnings(plate, warnings));
        }
        self
    }
}

impl CliFailure {
    /// `cli_report_config_substitutions` (`OrcaSlicer.cpp:520-534`) after the 3MF load (`:1992`):
    /// the stderr lines precede the family's own, and result.json records each substitution. The
    /// post_process rejection (`:1806`) comes before the report and carries none.
    pub(crate) fn with_config_substitutions(
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
}

/// The stderr lines of `cli_report_config_substitutions` (`OrcaSlicer.cpp:525-527`).
pub(crate) fn substitution_warnings(
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

impl FamilyFailure {
    pub(super) fn flush_and_exit(self) -> ! {
        #[cfg(target_os = "linux")]
        println!("run found error, return {}, exit...", self.return_code);
        #[cfg(not(target_os = "linux"))]
        println!("run found error, exit");
        eprint!("{}", self.stderr);
        #[cfg(target_os = "linux")]
        let _ = self.record_exit_reason();
        std::process::exit(self.return_code);
    }

    /// `record_exit_reson` with the family's fields (`OrcaSlicer.cpp:535-586`).
    #[cfg(target_os = "linux")]
    fn record_exit_reason(&self) -> Result<(), serde_json::Error> {
        let sliced_plates =
            self.critical_warning
                .as_ref()
                .map(|message| crate::result_json::SlicedPlateEntry {
                    id: self.plate_index,
                    triangle_count: 0,
                    warning_message: message.clone(),
                });
        crate::result_json::ResultJson {
            return_code: self.return_code,
            error_string: self.error_string,
            plate_index: self.plate_index,
            layer_height: self.layer_height,
            wall_loops: self.wall_loops,
            sparse_infill_density: self.sparse_infill_density,
            upward_machines: &self.upward_machines,
            sliced_plates: sliced_plates.as_slice(),
            warnings: &self.warnings,
        }
        .write(&self.output)
    }
}
