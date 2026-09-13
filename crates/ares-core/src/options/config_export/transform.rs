use crate::{OrcaFloat, ProjectSettings, SliceError};

use super::writer::ExportOverrides;

const MATRIX_ERROR: &str = "Flush volumes matrix do not match to the correct size!";

pub(super) fn transformed_for_export(
    source: &ProjectSettings,
    overrides: &ExportOverrides,
) -> Result<ProjectSettings, SliceError> {
    let mut transformed = source.clone();
    prepare_multi_extruder_cli_defaults(&mut transformed);
    apply_scarf_joint_seam(&mut transformed);
    apply_cli_oracle_state(&mut transformed, overrides);
    scale_flush_matrix(&mut transformed)?;
    Ok(transformed)
}

/// The CLI oracle's config export carries the RAW preset
/// `enable_prime_tower` for dual-nozzle BBL printers: the auto filament-map
/// recompute (`ToolOrdering.cpp:1288-1303`) runs for exactly those and
/// `Print::update_filament_maps_to_config` (`Print.cpp:3166`) rebuilds
/// `m_full_print_config` from `m_ori_full_print_config` (the raw preset
/// config). Every other printer's export reflects the `normalize_fdm_2`
/// used-filament disable (single-filament → 0).
fn apply_cli_oracle_state(settings: &mut ProjectSettings, overrides: &ExportOverrides) {
    let dual_nozzle_bbl = overrides.is_bbl && settings.project.print.nozzle_diameter.0.len() == 2;
    if dual_nozzle_bbl && let Some(raw) = overrides.enable_prime_tower {
        settings.process.print.enable_prime_tower = raw;
    }
}

/// `PrintApply.cpp:1148-1161`: `has_scarf_joint_seam` turns true when any
/// resolved `seam_slope_type` is not `none` (object, volume, layer-range or
/// the inherited default); the CLI export then carries the flag in the
/// `;CONFIG_BLOCK` comment.
fn apply_scarf_joint_seam(settings: &mut ProjectSettings) {
    use crate::ProcessSeamScarfType;
    if settings.process.region.seam_slope_type != ProcessSeamScarfType::None {
        settings.project.gcode.has_scarf_joint_seam = crate::OrcaBool(true);
    }
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
