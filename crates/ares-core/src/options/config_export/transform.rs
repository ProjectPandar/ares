use crate::{ProjectSettings, SliceError};

use super::writer::ExportOverrides;

const MATRIX_ERROR: &str = "Flush volumes matrix do not match to the correct size!";

pub(super) fn transformed_for_export(
    source: &ProjectSettings,
    overrides: &ExportOverrides,
) -> Result<ProjectSettings, SliceError> {
    let mut transformed = source.clone();
    apply_cli_oracle_state(&mut transformed, overrides);
    scale_flush_matrix(&mut transformed)?;
    Ok(transformed)
}

/// `Print::update_filament_maps_to_config` (`Print.cpp:3166-3170`) only
/// rebuilds `m_full_print_config` from the RAW `m_ori_full_print_config`
/// when the re-derived map actually CHANGED — the raw
/// `enable_prime_tower` resurfaces exactly then (H2D: map 1→2 keeps the
/// raw 1; X2D: map unchanged 1 exports the normalize_fdm_2-disabled 0).
fn apply_cli_oracle_state(settings: &mut ProjectSettings, overrides: &ExportOverrides) {
    let map_changed = match overrides.raw_filament_map_token.as_deref() {
        Some(raw) => {
            let derived = &settings.project.gcode.filament_map.0;
            let raw_list = raw
                .split(';')
                .filter_map(|token| token.trim().parse::<i32>().ok())
                .map(crate::OrcaInt)
                .collect::<Vec<_>>();
            raw_list != *derived
                && !(raw_list.len() == 1
                    && derived.len() > 1
                    && derived.iter().all(|value| value == &raw_list[0]))
        }
        None => false,
    };
    if map_changed && let Some(raw) = overrides.enable_prime_tower {
        settings.process.print.enable_prime_tower = raw;
    }
}

fn scale_flush_matrix(settings: &mut ProjectSettings) -> Result<(), SliceError> {
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
