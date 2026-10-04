//! Public `GCodeProcessorResult` move data (`GCodeProcessor.hpp:30-44`
//! `EMoveType`, `:184-218` `MoveVertex`), recorded by the processor pass
//! that also stamps the exported G-code.

use crate::ExtrusionRole;

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
            time: 0.0,
            layer_id: 0,
            internal_only: false,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GCodeProcessorResult {
    pub moves: Vec<MoveVertex>,
    pub spiral_vase_mode: bool,
}

/// Exported G-code together with the processor result of the same pass.
#[derive(Clone, Debug, PartialEq)]
pub struct ProcessedGCode {
    pub gcode: Vec<u8>,
    pub result: GCodeProcessorResult,
}
