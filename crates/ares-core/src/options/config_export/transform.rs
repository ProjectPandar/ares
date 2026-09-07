use crate::{OrcaFloat, ProjectSettings, SliceError};

const MATRIX_ERROR: &str = "Flush volumes matrix do not match to the correct size!";

pub(super) fn transformed_for_export(
    source: &ProjectSettings,
) -> Result<ProjectSettings, SliceError> {
    let mut transformed = source.clone();
    prepare_multi_extruder_cli_defaults(&mut transformed);
    scale_flush_matrix(&mut transformed)?;
    Ok(transformed)
}

/// The OrcaSlicer CLI slicing path fills per-extruder defaults for
/// multi-extruder printers (`OrcaSlicer.cpp:5993-6022`, applied when
/// `filament_map_mode < Manual`): every extruder gets the `1#0|4#1`
/// AMS mapping, `flush_multiplier` resizes to the extruder count
/// (default fill 1.0), and the flush volumes matrix collapses to a
/// zero diagonal sized to the extruder count.
fn prepare_multi_extruder_cli_defaults(settings: &mut ProjectSettings) {
    use crate::ProjectFilamentMapMode;

    let count = settings.project.print.nozzle_diameter.0.len();
    if count <= 1
        || !settings
            .project
            .gcode
            .extruder_ams_count
            .0
            .iter()
            .all(|value| value.is_empty())
        || settings.project.gcode.filament_map_mode == ProjectFilamentMapMode::Manual
    {
        return;
    }
    settings.project.gcode.extruder_ams_count.0 = vec!["1#0|4#1".to_owned(); count];
    settings
        .project
        .print
        .flush_multiplier
        .0
        .resize(count, OrcaFloat(1.0));
    // The zero-diagonal collapse is only pinned for single-filament
    // prints; multi-filament flush matrices come from the CLI's
    // `get_flush_volumes_matrix` computation (out of scope here).
    if settings.filament.gcode.filament_colour.0.len() == 1 {
        settings.project.print.flush_volumes_matrix.0 = vec![0.0; count];
    }
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
