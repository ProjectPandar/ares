//! Public `GCodeProcessorResult` move data (`GCodeProcessor.hpp:30-44`
//! `EMoveType`, `:184-218` `MoveVertex`), recorded by the processor pass
//! that also stamps the exported G-code.

mod chunked_moves;
pub(crate) mod printable_area;
pub use chunked_moves::ChunkedMoves;

use crate::ExtrusionRole;
pub use crate::project_slice::gcode_emit::processor::process_gcode;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum EMoveType {
    Noop,
    Retract,
    Unretract,
    Seam,
    ToolChange,
    ColorChange,
    PausePrint,
    CustomGCode,
    Travel,
    Wipe,
    Extrude,
}

/// Upstream `GCodeProcessorResult::MoveVertex` restricted to the fields this
/// slice records; `time` is the normal-mode `time[0]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MoveVertex {
    /// 1-based line of the exported G-code (`m_last_line_id`).
    pub gcode_id: u32,
    pub move_type: EMoveType,
    pub extrusion_role: ExtrusionRole,
    pub extruder_id: u8,
    pub cp_color_id: u8,
    pub position: [f32; 3],
    pub delta_extruder: f32,
    pub feedrate: f32,
    pub width: f32,
    pub height: f32,
    pub mm3_per_mm: f32,
    pub travel_dist: f32,
    pub fan_speed: f32,
    pub temperature: f32,
    /// ORCA preview attributes (`GCodeProcessor.hpp:194-217`), Normal mode.
    pub pressure_advance: f32,
    pub acceleration: f32,
    pub jerk: f32,
    /// Normal-mode planner speed (`GCodeProcessor.hpp:194`, mm/s).
    pub actual_feedrate: f32,
    pub time: f32,
    pub layer_id: u32,
    /// Actual-speed vertex inserted by `calculate_time` (`:5962-5997`).
    pub internal_only: bool,
}

impl Default for MoveVertex {
    /// The dummy first move (`GCodeProcessor::initialize_result_moves`).
    fn default() -> Self {
        Self {
            gcode_id: 0,
            move_type: EMoveType::Noop,
            extrusion_role: ExtrusionRole::None,
            extruder_id: 0,
            cp_color_id: 0,
            position: [0.0; 3],
            delta_extruder: 0.0,
            feedrate: 0.0,
            width: 0.0,
            height: 0.0,
            mm3_per_mm: 0.0,
            travel_dist: 0.0,
            fan_speed: 0.0,
            temperature: 0.0,
            pressure_advance: 0.0,
            acceleration: 0.0,
            jerk: 0.0,
            actual_feedrate: 0.0,
            time: 0.0,
            layer_id: 0,
            internal_only: false,
        }
    }
}

/// `GCodeProcessorResult::print_statistics` — the extruded-volume
/// buckets the viewer's Filament/ColorPrint legend reads
/// (`GCodeProcessor.hpp:46-78`, `UsedFilaments::process_*_cache`,
/// `GCodeProcessor.cpp:3895-3911, 5931-5944, 6031-6037`). Volumes are
/// mm³ dense-indexed by filament id (zero for unused filaments);
/// upstream's sparse `std::map` is equivalent because every legend
/// lookup probes `find(id)`.
///
/// `total_used_filament` is `PrintStatistics::total_used_filament`
/// (`GCode.cpp:1914-1937`): Σ volume / (π·(d/2)²) in mm.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize)]
pub struct PrintEstimatedStatistics {
    /// Nonzero chronological model caches (`GCodeProcessor.cpp:1549-1555`).
    pub volumes_per_color_change: Vec<f64>,
    /// `UsedFilaments::filaments_per_role`: meters and grams by extrusion
    /// role (`GCodeProcessor.cpp:1621-1639,6036`).
    pub used_filaments_per_role: std::collections::BTreeMap<ExtrusionRole, (f64, f64)>,
    /// `UsedFilaments::model_volumes_per_filament` →
    /// `model_volumes_per_extruder`.
    pub model_volumes_per_extruder: Vec<f64>,
    /// `UsedFilaments::wipe_tower_volumes_per_filament` →
    /// `wipe_tower_volumes_per_extruder`.
    pub wipe_tower_volumes_per_extruder: Vec<f64>,
    /// `UsedFilaments::support_volumes_per_filament` →
    /// `support_volumes_per_extruder`.
    pub support_volumes_per_extruder: Vec<f64>,
    /// `UsedFilaments::total_volumes_per_filament` →
    /// `total_volumes_per_extruder`.
    pub total_volumes_per_extruder: Vec<f64>,
    /// `UsedFilaments::flush_per_filament` → `flush_per_filament`.
    pub flush_per_filament: Vec<f64>,
    /// `PrintStatistics::total_used_filament` (mm).
    pub total_used_filament: f64,
    /// `modes[Normal].time`: double machine time returned without narrowing
    /// (`GCodeProcessor.hpp:96,843`, `GCodeProcessor.cpp:4032-4034`).
    pub normal_time: f64,
    /// `modes[Normal].prepare_time`: f32 sum of the `flags.prepare_stage`
    /// block times (`GCodeProcessor.cpp:475-476`), copied by
    /// `update_estimated_times_stats` (`:6016-6022`).
    pub normal_prepare_time: f32,
}

/// `CustomGCode::Type` (`CustomGCode.hpp:14-22`).
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
#[repr(u8)]
pub enum CustomGCodeType {
    ColorChange,
    PausePrint,
    ToolChange,
    Template,
    Custom,
    Unknown,
}

/// Processor custom event (`CustomGCode.hpp:24-43`), with one-based tool id.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct CustomGCodeItem {
    pub print_z: f64,
    pub kind: CustomGCodeType,
    pub extruder: u32,
    pub color: String,
    pub extra: String,
}

/// Normal-mode `get_custom_gcode_times` (GCodeProcessor.cpp:2737-2750).
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
pub struct CustomGCodeTime {
    pub kind: CustomGCodeType,
    pub duration: f32,
    pub remaining: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GCodeProcessorResult {
    pub moves: ChunkedMoves,
    pub spiral_vase_mode: bool,
    /// `GCodeProcessorResult::custom_gcode_per_print_z` (`:260`).
    pub custom_gcode_per_print_z: Vec<CustomGCodeItem>,
    /// Normal-mode `PrintEstimatedStatistics::Mode::custom_gcode_times`.
    pub custom_gcode_times: Vec<CustomGCodeTime>,
    /// `GCodeProcessorResult::print_statistics` (`GCodeProcessor.hpp:118`).
    pub print_statistics: PrintEstimatedStatistics,
    /// `GCodeProcessorResult::extruder_colors` — `filament_colour`
    /// config values, padded to `filaments_count` and defaulted to
    /// `#FF8000` (`GCodeProcessor::apply_config`,
    /// `GCodeProcessor.cpp:2321-2343`).
    pub extruder_colors: Vec<String>,
    /// `GCodeProcessorResult::filament_diameters` (`GCodeProcessor.hpp:252`).
    pub filament_diameters: Vec<f32>,
    /// `GCodeProcessorResult::filament_densities` (`GCodeProcessor.hpp:254`).
    pub filament_densities: Vec<f32>,
}

impl GCodeProcessorResult {
    /// The `filament_diameters`/`filament_densities` slice of
    /// `GCodeProcessor::apply_config`: both are sized to
    /// `filament_count = filament_diameter.size()` (`GCodeProcessor.cpp:2033,
    /// 2054-2056`) and slot `i` reads `get_at(i)` as float (`:2081, 2083`),
    /// which falls back to the first value (`Config.hpp:624-628`). The
    /// `reset()` defaults (`:1672-1674`) are always overwritten here, so a
    /// key missing from the config reads its `PrintConfig` default instead
    /// (`filament_diameter` `{ 1.75 }`, `filament_density` `{ 0. }`,
    /// `PrintConfig.cpp:2606-2611, 2864-2870`), as resolved by the options.
    pub(crate) fn apply_filament_config(&mut self, filament: &crate::FilamentGCodeSourceOptions) {
        let diameters = &filament.filament_diameter.0;
        let densities = &filament.filament_density.0;
        let get_at =
            |values: &[crate::OrcaFloat], id: usize| values.get(id).unwrap_or(&values[0]).0 as f32;
        self.filament_diameters = (0..diameters.len())
            .map(|id| get_at(diameters, id))
            .collect();
        self.filament_densities = (0..diameters.len())
            .map(|id| get_at(densities, id))
            .collect();
    }
}

/// The `PrintStatistics` totals (`Print.hpp:789-816`) that
/// `DoExport::update_print_estimated_stats` writes after
/// `GCodeProcessor::finalize` (`GCode.cpp:1904-1940, 2176`).
#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize)]
pub struct PrintStatistics {
    /// mm³, every `total_volumes_per_extruder` entry.
    pub total_extruded_volume: f64,
    /// mm, writer extruders only.
    pub total_used_filament: f64,
    /// g, writer extruders only.
    pub total_weight: f64,
    /// Filament cost of writer extruders plus `time_cost` per normal-mode hour.
    pub total_cost: f64,
}

/// Exported G-code together with the processor result of the same pass.
#[derive(Clone, Debug, PartialEq)]
pub struct ProcessedGCode {
    pub gcode: Vec<u8>,
    /// PrintObject step warnings from the slice, not GCodeProcessor data.
    pub warnings: Vec<crate::PrintWarning>,
    pub result: GCodeProcessorResult,
    /// `Print::print_statistics()` after export.
    pub print_statistics: PrintStatistics,
}
