use crate::{ProjectSettings, SliceError};

const MATRIX_ERROR: &str = "Flush volumes matrix do not match to the correct size!";

pub(super) fn transformed_for_export(
    source: &ProjectSettings,
) -> Result<ProjectSettings, SliceError> {
    let mut transformed = source.clone();
    scale_flush_matrix(&mut transformed)?;
    Ok(transformed)
}

fn scale_flush_matrix(settings: &mut ProjectSettings) -> Result<(), SliceError> {
    let multipliers = &settings.project.print.flush_multiplier.0;
    let head_count = multipliers.len();
    if head_count == 0 {
        return Err(matrix_error());
    }

    let filament_count = settings.filament.gcode.filament_colour.0.len();
    let matrix = &mut settings.project.print.flush_volumes_matrix.0;
    let expected = filament_count
        .checked_mul(filament_count)
        .and_then(|count| count.checked_mul(head_count));
    if expected == Some(matrix.len()) {
        let segment_len = matrix.len() / head_count;
        for (head, multiplier) in multipliers.iter().enumerate() {
            let start = head * segment_len;
            let end = start + segment_len;
            for value in &mut matrix[start..end] {
                *value = (*value * multiplier.0).round();
            }
        }
        Ok(())
    } else if filament_count == 1 {
        Ok(())
    } else {
        Err(matrix_error())
    }
}

fn matrix_error() -> SliceError {
    SliceError::InvalidInput(MATRIX_ERROR.to_owned())
}
