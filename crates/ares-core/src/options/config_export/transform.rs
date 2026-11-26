use crate::options::FlatMatrix;
use crate::{ProjectSettings, SliceError};

const MATRIX_ERROR: &str = "Flush volumes matrix do not match to the correct size!";

pub(super) fn scaled_flush_matrix(
    settings: &ProjectSettings,
) -> Result<Option<FlatMatrix>, SliceError> {
    let multipliers = match settings.project.print.prime_volume_mode {
        crate::ProjectPrimeVolumeMode::Fast => &settings.project.print.flush_multiplier_fast.0,
        crate::ProjectPrimeVolumeMode::Default | crate::ProjectPrimeVolumeMode::Saving => {
            &settings.project.print.flush_multiplier.0
        }
    };
    let head_count = multipliers.len();
    if head_count == 0 {
        return Err(matrix_error());
    }

    let filament_count = settings.filament.gcode.filament_colour.0.len();
    let source = &settings.project.print.flush_volumes_matrix.0;
    let expected = filament_count
        .checked_mul(filament_count)
        .and_then(|count| count.checked_mul(head_count));
    if expected == Some(source.len()) {
        let mut matrix = source.clone();
        let segment_len = matrix.len() / head_count;
        for (head, multiplier) in multipliers.iter().enumerate() {
            let start = head * segment_len;
            let end = start + segment_len;
            for value in &mut matrix[start..end] {
                *value = (*value * multiplier.0).round();
            }
        }
        Ok(Some(FlatMatrix(matrix)))
    } else if filament_count == 1 {
        Ok(None)
    } else {
        Err(matrix_error())
    }
}

fn matrix_error() -> SliceError {
    SliceError::InvalidInput(MATRIX_ERROR.to_owned())
}
