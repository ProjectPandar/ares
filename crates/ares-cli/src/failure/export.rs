use std::path::Path;

use super::{CliFailure, FamilyFailure};

impl CliFailure {
    pub(crate) fn project_export(output: &Path, invalid_arguments: bool, detail: String) -> Self {
        let (return_code, message) = if invalid_arguments {
            (-2, "Invalid parameters to the slicer.")
        } else {
            (-13, "Failed exporting 3mf files.")
        };
        let stderr = if invalid_arguments {
            format!("{message} {detail}\n")
        } else {
            format!(
                "Project export to {} failed\n{message} {detail}\n",
                output.display()
            )
        };
        Self::Family(Box::new(FamilyFailure {
            return_code,
            #[cfg(target_os = "linux")]
            plate_index: 0,
            #[cfg(target_os = "linux")]
            layer_height: 0.0,
            #[cfg(target_os = "linux")]
            wall_loops: 0,
            #[cfg(target_os = "linux")]
            sparse_infill_density: 0.0,
            #[cfg(target_os = "linux")]
            error_string: message.to_owned(),
            upward_machines: Vec::new(),
            #[cfg(target_os = "linux")]
            warnings: Vec::new(),
            #[cfg(target_os = "linux")]
            critical_warning: None,
            #[cfg(target_os = "linux")]
            completed: Vec::new(),
            stderr,
            #[cfg(target_os = "linux")]
            output: output.to_owned(),
        }))
    }
}
