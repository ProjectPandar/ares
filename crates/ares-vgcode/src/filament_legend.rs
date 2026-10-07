// Ports the used filament data of AGPL-licensed OrcaSlicer
// `src/slic3r/GUI/GCodeViewer.cpp` `render_legend` for the ColorPrint view.

use crate::{Color, ViewerImpl};
use ares_core::gcode_processor::GCodeProcessorResult;

/// `displayed_columns` in header order (`GCodeViewer.cpp:3749-3767`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilamentColumn {
    Model,
    Support,
    Flushed,
    Tower,
    Total,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FilamentRow {
    pub extruder_id: u8,
    pub color: Color,
    /// mm³ for each displayed volume column (Total is computed after unit conversion).
    pub volumes: Vec<f64>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FilamentLegend {
    pub columns: Vec<FilamentColumn>,
    pub rows: Vec<FilamentRow>,
    /// GCodeProcessorResult metadata for display-only m/g conversion.
    pub filament_diameters: Vec<f32>,
    pub filament_densities: Vec<f32>,
}

/// The ColorPrint legend of `render_legend` (`GCodeViewer.cpp:3563-3625,
/// 3749-3767, 4044-4094`). Dense statistics omit a bucket with zero volume,
/// matching `UsedFilaments::process_*_cache` (`GCodeProcessor.cpp:1558-1600`).
/// Unit conversion and formatting belong to the display layer.
pub fn filament_legend(viewer: &ViewerImpl, result: &GCodeProcessorResult) -> FilamentLegend {
    let statistics = &result.print_statistics;
    let used = viewer.get_used_extruders_ids();
    let shown: Vec<_> = [
        (
            FilamentColumn::Model,
            &statistics.model_volumes_per_extruder,
        ),
        (
            FilamentColumn::Support,
            &statistics.support_volumes_per_extruder,
        ),
        (FilamentColumn::Flushed, &statistics.flush_per_filament),
        (
            FilamentColumn::Tower,
            &statistics.wipe_tower_volumes_per_extruder,
        ),
    ]
    .into_iter()
    .filter(|(_, volumes)| {
        used.iter()
            .any(|&id| volumes.get(usize::from(id)).is_some_and(|v| *v != 0.0))
    })
    .collect();
    let mut columns: Vec<_> = shown.iter().map(|(column, _)| *column).collect();
    if columns
        .iter()
        .any(|column| *column != FilamentColumn::Model)
    {
        columns.push(FilamentColumn::Total);
    }
    let rows = used
        .into_iter()
        .map(|id| FilamentRow {
            extruder_id: id,
            color: viewer.get_tool_colors()[usize::from(id)],
            volumes: shown
                .iter()
                .map(|(_, volumes)| volumes.get(usize::from(id)).copied().unwrap_or_default())
                .collect(),
        })
        .collect();
    FilamentLegend {
        columns,
        rows,
        filament_diameters: result.filament_diameters.clone(),
        filament_densities: result.filament_densities.clone(),
    }
}
