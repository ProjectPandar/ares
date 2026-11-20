use super::*;

impl CliFailure {
    pub(crate) fn assembly(
        error: ares_core::AssemblyError,
        #[cfg(target_os = "linux")] output: &Path,
    ) -> Self {
        Self::Family(Box::new(FamilyFailure {
            return_code: error.return_code(),
            #[cfg(target_os = "linux")]
            plate_index: 0,
            #[cfg(target_os = "linux")]
            layer_height: 0.0,
            #[cfg(target_os = "linux")]
            wall_loops: 0,
            #[cfg(target_os = "linux")]
            sparse_infill_density: 0.0,
            #[cfg(target_os = "linux")]
            error_string: error.error_string().to_owned(),
            upward_machines: Vec::new(),
            #[cfg(target_os = "linux")]
            warnings: Vec::new(),
            #[cfg(target_os = "linux")]
            plate_warning: None,
            #[cfg(target_os = "linux")]
            completed: Vec::new(),
            stderr: format!("{error}\n"),
            #[cfg(target_os = "linux")]
            output: output.to_owned(),
        }))
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

impl CliFailure {
    pub(crate) fn missing_default_filament(#[cfg(target_os = "linux")] output: &Path) -> Self {
        // OrcaSlicer.cpp:2411–2415 reports CLI_CONFIG_FILE_ERROR before slicing.
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
            error_string: "The input preset file is invalid and can not be parsed.".to_owned(),
            upward_machines: Vec::new(),
            #[cfg(target_os = "linux")]
            warnings: Vec::new(),
            #[cfg(target_os = "linux")]
            plate_warning: None,
            #[cfg(target_os = "linux")]
            completed: Vec::new(),
            stderr: String::new(),
            #[cfg(target_os = "linux")]
            output: output.to_owned(),
        }))
    }

    pub(crate) fn strict_no_check(#[cfg(target_os = "linux")] output: &Path) -> Self {
        Self::Family(Box::new(FamilyFailure {
            return_code: -2,
            #[cfg(target_os = "linux")]
            plate_index: 0,
            #[cfg(target_os = "linux")]
            layer_height: 0.0,
            #[cfg(target_os = "linux")]
            wall_loops: 0,
            #[cfg(target_os = "linux")]
            sparse_infill_density: 0.0,
            #[cfg(target_os = "linux")]
            error_string: "Invalid parameters to the slicer.".to_owned(),
            upward_machines: Vec::new(),
            #[cfg(target_os = "linux")]
            warnings: Vec::new(),
            #[cfg(target_os = "linux")]
            plate_warning: None,
            #[cfg(target_os = "linux")]
            completed: Vec::new(),
            stderr: "--strict cannot be combined with --no-check\n".to_owned(),
            #[cfg(target_os = "linux")]
            output: output.to_owned(),
        }))
    }

    pub(crate) fn statistics_write(
        path: &Path,
        plate: u32,
        #[cfg(target_os = "linux")] output: &Path,
    ) -> Self {
        Self::Family(Box::new(FamilyFailure {
            return_code: -25,
            #[cfg(target_os = "linux")]
            plate_index: plate,
            #[cfg(target_os = "linux")]
            layer_height: 0.0,
            #[cfg(target_os = "linux")]
            wall_loops: 0,
            #[cfg(target_os = "linux")]
            sparse_infill_density: 0.0,
            #[cfg(target_os = "linux")]
            error_string: "Failed exporting the G-code statistics.".to_owned(),
            upward_machines: Vec::new(),
            #[cfg(target_os = "linux")]
            warnings: Vec::new(),
            #[cfg(target_os = "linux")]
            plate_warning: None,
            #[cfg(target_os = "linux")]
            completed: Vec::new(),
            stderr: format!(
                "plate {plate}: Failed to write the G-code statistics to {}.\n",
                path.display()
            ),
            #[cfg(target_os = "linux")]
            output: output.to_owned(),
        }))
    }

    pub(crate) fn invalid_stl_input(
        input: &Path,
        #[cfg(target_os = "linux")] output: &Path,
    ) -> Self {
        // Model::read_from_file throws the model-load detail (Model.cpp:493–498);
        // the CLI adds the input path and returns CLI_DATA_FILE_ERROR (OrcaSlicer.cpp:1999–2002).
        Self::Family(Box::new(FamilyFailure {
            return_code: -6,
            #[cfg(target_os = "linux")]
            plate_index: 0,
            #[cfg(target_os = "linux")]
            layer_height: 0.0,
            #[cfg(target_os = "linux")]
            wall_loops: 0,
            #[cfg(target_os = "linux")]
            sparse_infill_density: 0.0,
            #[cfg(target_os = "linux")]
            error_string: "The input model file to the slicer can not be parsed.".to_owned(),
            upward_machines: Vec::new(),
            #[cfg(target_os = "linux")]
            warnings: Vec::new(),
            #[cfg(target_os = "linux")]
            plate_warning: None,
            #[cfg(target_os = "linux")]
            completed: Vec::new(),
            stderr: format!("{}: Loading of a model file failed.\n", input.display()),
            #[cfg(target_os = "linux")]
            output: output.to_owned(),
        }))
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
    pub(crate) fn invalid_slice_selection(#[cfg(target_os = "linux")] output: &Path) -> Self {
        Self::slice_action_failure(
            #[cfg(target_os = "linux")]
            output,
            #[cfg(target_os = "linux")]
            0,
            -2,
            #[cfg(target_os = "linux")]
            "Invalid parameters to the slicer.",
            String::new(),
        )
    }

    pub(crate) fn slice_export_error(
        #[cfg(target_os = "linux")] output: &Path,
        #[cfg(target_os = "linux")] plate: u32,
        error: io::Error,
    ) -> Self {
        Self::slice_action_failure(
            #[cfg(target_os = "linux")]
            output,
            #[cfg(target_os = "linux")]
            plate,
            -100,
            #[cfg(target_os = "linux")]
            "Failed slicing the model. Please verify the slicing of all plates on Orca Slicer before uploading.",
            format!("{error}\n"),
        )
    }

    fn slice_action_failure(
        #[cfg(target_os = "linux")] output: &Path,
        #[cfg(target_os = "linux")] plate_index: u32,
        return_code: i32,
        #[cfg(target_os = "linux")] error_string: &'static str,
        stderr: String,
    ) -> Self {
        Self::Family(Box::new(FamilyFailure {
            return_code,
            #[cfg(target_os = "linux")]
            plate_index,
            #[cfg(target_os = "linux")]
            layer_height: 0.0,
            #[cfg(target_os = "linux")]
            wall_loops: 0,
            #[cfg(target_os = "linux")]
            sparse_infill_density: 0.0,
            #[cfg(target_os = "linux")]
            error_string: error_string.to_owned(),
            upward_machines: Vec::new(),
            #[cfg(target_os = "linux")]
            warnings: Vec::new(),
            #[cfg(target_os = "linux")]
            plate_warning: None,
            #[cfg(target_os = "linux")]
            completed: Vec::new(),
            stderr,
            #[cfg(target_os = "linux")]
            output: output.to_owned(),
        }))
    }

    #[cfg(target_os = "linux")]
    pub(crate) fn with_plate_batch(
        mut self,
        plate: u32,
        report: ares_core::SlicedPlate,
        completed: Vec<(u32, ares_core::SlicedPlate)>,
    ) -> Self {
        if let Self::Family(family) = &mut self {
            if family.return_code == -50 || family.return_code == -100 {
                family.layer_height = report.layer_height;
                family.wall_loops = report.wall_loops;
                family.sparse_infill_density = report.sparse_infill_density;
            }
            for (id, report) in completed {
                family
                    .completed
                    .push(crate::result_json::SlicedPlateEntry::from_plate(
                        id, &report,
                    ));
                family
                    .warnings
                    .extend(crate::result_json::slicing_warnings(id, report.warnings));
            }
            family
                .warnings
                .extend(crate::result_json::slicing_warnings(plate, report.warnings));
        }
        self
    }
}

impl FamilyFailure {
    pub(super) fn flush_and_exit(self, #[cfg(target_os = "linux")] strict_mode: bool) -> ! {
        #[cfg(target_os = "linux")]
        println!("run found error, return {}, exit...", self.return_code);
        #[cfg(not(target_os = "linux"))]
        println!("run found error, exit");
        eprint!("{}", self.stderr);
        #[cfg(target_os = "linux")]
        let _ = self.record_exit_reason(strict_mode);
        std::process::exit(self.return_code);
    }

    /// `record_exit_reson` with the family's fields (`OrcaSlicer.cpp:535-586`).
    #[cfg(target_os = "linux")]
    fn record_exit_reason(&self, strict_mode: bool) -> Result<(), serde_json::Error> {
        let plate_warning =
            self.plate_warning
                .as_ref()
                .map(|message| crate::result_json::SlicedPlateEntry {
                    id: self.plate_index,
                    triangle_count: 0,
                    warning_message: message.clone(),
                });
        let sliced_plates = self
            .completed
            .iter()
            .cloned()
            .chain(plate_warning)
            .collect::<Vec<_>>();
        crate::result_json::ResultJson {
            return_code: self.return_code,
            error_string: &self.error_string,
            plate_index: self.plate_index,
            layer_height: self.layer_height,
            wall_loops: self.wall_loops,
            sparse_infill_density: self.sparse_infill_density,
            strict_mode,
            upward_machines: &self.upward_machines,
            sliced_plates: sliced_plates.as_slice(),
            warnings: &self.warnings,
        }
        .write(&self.output)
    }
}
