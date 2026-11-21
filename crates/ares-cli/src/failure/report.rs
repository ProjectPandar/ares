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
            downward_machines: &self.downward_machines,
            sliced_plates: sliced_plates.as_slice(),
            warnings: &self.warnings,
        }
        .write(&self.output)
    }
}
