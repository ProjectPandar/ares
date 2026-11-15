//! `DoExport::update_print_estimated_stats` (`GCode.cpp:1904-1940`): the
//! `PrintStatistics` totals read by the preview Summary legend. It shares
//! `total_volumes_per_extruder` and `modes[Normal].time` with
//! `material_statistics.rs`, but ports a different upstream function:
//! per-filament values are `Extruder::filament_*()` doubles
//! (`Extruder.cpp:148-161`), `time_cost` is charged per normal-mode hour,
//! and ids missing from `m_writer.extruders()` add only to the extruded
//! volume.
use crate::{OrcaFloats, PrintEstimatedStatistics, PrintStatistics, ProjectSettings};

pub(super) fn update_print_estimated_stats(
    statistics: &PrintEstimatedStatistics,
    writer_extruders: &[usize],
    settings: &ProjectSettings,
) -> PrintStatistics {
    let filament = &settings.filament.gcode;
    let normal_print_time = statistics.normal_time;
    let mut total = PrintStatistics::default();
    for (id, &volume) in statistics.total_volumes_per_extruder.iter().enumerate() {
        total.total_extruded_volume += volume;
        if !writer_extruders.contains(&id) {
            continue;
        }
        let radius = 0.5 * get_at(&filament.filament_diameter, id);
        let s = std::f64::consts::PI * (radius * radius);
        let weight = volume * get_at(&filament.filament_density, id) * 0.001;
        total.total_used_filament += volume / s;
        total.total_weight += weight;
        total.total_cost += weight * get_at(&filament.filament_cost, id) * 0.001;
    }
    total.total_cost += settings.printer.gcode.time_cost.0 * (normal_print_time / 3600.0);
    total
}

/// `ConfigOptionVector::get_at` (`Config.hpp:624-628`).
fn get_at(values: &OrcaFloats, id: usize) -> f64 {
    values.0.get(id).unwrap_or(&values.0[0]).0
}
