use crate::SliceError;

/// `OrcaSlicer.cpp:1671–1748`: the input-list checks precede opening a 3MF.
pub struct CliObjectAssignment {
    counts: [usize; 2],
}

impl CliObjectAssignment {
    pub fn new(clone_counts: &[i32], filament_ids: &[i32]) -> Self {
        Self {
            counts: [clone_counts.len(), filament_ids.len()],
        }
    }

    pub fn is_requested(&self) -> bool {
        self.counts != [0, 0]
    }

    pub fn validate_prerequisites(&self, has_filament_list: bool) -> Result<(), SliceError> {
        // The saved-project command accepts one model file.
        for (name, count) in ["clone_objects", "loaded_filament_ids"]
            .into_iter()
            .zip(self.counts)
        {
            match count {
                0 => (),
                1 if has_filament_list => (),
                1 => {
                    return Err(SliceError::InvalidCliObjectAssignment(format!(
                        "{name} should be used with load_filaments together"
                    )));
                }
                _ => {
                    return Err(SliceError::InvalidCliObjectAssignment(format!(
                        "{name} size {count} should be the same with input files size 1"
                    )));
                }
            }
        }
        Ok(())
    }

    pub fn validate_project_input(&self) -> Result<(), SliceError> {
        if self.is_requested() {
            return Err(SliceError::InvalidCliObjectAssignment(
                "can not load 3mf when set loaded_filament_ids or clone_objects".into(),
            ));
        }
        Ok(())
    }
}
