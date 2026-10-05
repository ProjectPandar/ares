// Ports AGPL-licensed OrcaSlicer `src/slic3r/GUI/LibVGCode/LibVGCodeWrapper.cpp`
// `convert(const GCodeProcessorResult&, ...)` (`:191-268`) and the role / move
// type / color conversions it uses (`:46-146`).

use ares_core::ExtrusionRole;
use ares_core::gcode_processor::{EMoveType, GCodeProcessorResult, MoveVertex};

use crate::{
    Color, DUMMY_COLOR, GCodeExtrusionRole, GCodeInputData, MoveType, OptionType, PathVertex,
    TimeMode, move_type_to_option,
};

/// `libvgcode::convert(result, str_tool_colors, str_color_print_colors, viewer)`.
pub fn from_processor_result(
    result: &GCodeProcessorResult,
    tool_colors: &[String],
    color_print_colors: &[String],
) -> GCodeInputData {
    let color_print_colors = if color_print_colors.is_empty() {
        tool_colors
    } else {
        color_print_colors
    };
    let moves = &result.moves;
    let mut vertices = Vec::with_capacity(2 * moves.len());
    for pair in moves.windows(2) {
        let [prev, curr] = pair else { unreachable!() };
        let move_type = move_type(curr.move_type);
        let starts_path = matches!(
            move_type_to_option(move_type),
            None | Some(OptionType::Travels | OptionType::Wipes)
        ) && (vertices.is_empty()
            || prev.move_type != curr.move_type
            || prev.extrusion_role != curr.extrusion_role
            // ORCA: flow rate changes start a new path.
            || prev.mm3_per_mm != curr.mm3_per_mm);
        if starts_path {
            // Phantom vertex at the previous position so libvgcode detects the
            // path start; its times stay zero.
            vertices.push(PathVertex {
                position: prev.position,
                times: [0.0; TimeMode::COUNT],
                ..vertex(curr, move_type)
            });
        }
        vertices.push(vertex(curr, move_type));
    }
    GCodeInputData {
        spiral_vase_mode: result.spiral_vase_mode,
        vertices,
        tools_colors: tool_colors.iter().map(|color| decode(color)).collect(),
        color_print_colors: color_print_colors
            .iter()
            .map(|color| decode(color))
            .collect(),
    }
}

/// Fields deferred by M853 (`actual_feedrate`, stealth time, pressure
/// advance, acceleration, jerk) stay zero.
fn vertex(curr: &MoveVertex, move_type: MoveType) -> PathVertex {
    PathVertex {
        position: curr.position,
        height: curr.height,
        width: curr.width,
        feedrate: curr.feedrate,
        actual_feedrate: 0.0,
        mm3_per_mm: curr.mm3_per_mm,
        fan_speed: curr.fan_speed,
        temperature: curr.temperature,
        role: role(curr.extrusion_role),
        move_type,
        gcode_id: curr.gcode_id,
        layer_id: curr.layer_id,
        extruder_id: curr.extruder_id,
        color_id: curr.cp_color_id,
        times: [curr.time, 0.0],
        layer_duration: 0.0,
        pressure_advance: 0.0,
        acceleration: 0.0,
        jerk: 0.0,
    }
}

const fn move_type(move_type: EMoveType) -> MoveType {
    match move_type {
        EMoveType::Noop => MoveType::Noop,
        EMoveType::Retract => MoveType::Retract,
        EMoveType::Unretract => MoveType::Unretract,
        EMoveType::Seam => MoveType::Seam,
        EMoveType::ToolChange => MoveType::ToolChange,
        EMoveType::ColorChange => MoveType::ColorChange,
        EMoveType::PausePrint => MoveType::PausePrint,
        EMoveType::CustomGCode => MoveType::CustomGCode,
        EMoveType::Travel => MoveType::Travel,
        EMoveType::Wipe => MoveType::Wipe,
        EMoveType::Extrude => MoveType::Extrude,
    }
}

const fn role(role: ExtrusionRole) -> GCodeExtrusionRole {
    match role {
        ExtrusionRole::None => GCodeExtrusionRole::None,
        ExtrusionRole::Perimeter => GCodeExtrusionRole::Perimeter,
        ExtrusionRole::ExternalPerimeter => GCodeExtrusionRole::ExternalPerimeter,
        ExtrusionRole::OverhangPerimeter => GCodeExtrusionRole::OverhangPerimeter,
        ExtrusionRole::InternalInfill => GCodeExtrusionRole::InternalInfill,
        ExtrusionRole::SolidInfill => GCodeExtrusionRole::SolidInfill,
        ExtrusionRole::TopSolidInfill => GCodeExtrusionRole::TopSolidInfill,
        ExtrusionRole::BottomSurface => GCodeExtrusionRole::BottomSurface,
        ExtrusionRole::Ironing => GCodeExtrusionRole::Ironing,
        ExtrusionRole::BridgeInfill => GCodeExtrusionRole::BridgeInfill,
        ExtrusionRole::InternalBridgeInfill => GCodeExtrusionRole::InternalBridgeInfill,
        ExtrusionRole::GapFill => GCodeExtrusionRole::GapFill,
        ExtrusionRole::Skirt => GCodeExtrusionRole::Skirt,
        ExtrusionRole::Brim => GCodeExtrusionRole::Brim,
        ExtrusionRole::SupportMaterial => GCodeExtrusionRole::SupportMaterial,
        ExtrusionRole::SupportMaterialInterface => GCodeExtrusionRole::SupportMaterialInterface,
        ExtrusionRole::SupportTransition => GCodeExtrusionRole::SupportTransition,
        ExtrusionRole::WipeTower => GCodeExtrusionRole::WipeTower,
        ExtrusionRole::Custom => GCodeExtrusionRole::Custom,
        ExtrusionRole::Mixed => GCodeExtrusionRole::Mixed,
    }
}

/// `convert(const std::string&)` over `decode_color` (`Color.cpp:332-367`):
/// `#RRGGBB` / `#RRGGBBAA`, invalid digit pairs stay black, and the ORCA
/// minimum brightness clamps each channel to 48.
fn decode(color: &str) -> Color {
    let bytes = color.as_bytes();
    if !matches!(bytes.len(), 7 | 9) || bytes[0] != b'#' {
        return DUMMY_COLOR;
    }
    let digit = |c: u8| (c as char).to_digit(16).map(|d| d as u8);
    std::array::from_fn(|i| {
        let value = match (digit(bytes[1 + 2 * i]), digit(bytes[2 + 2 * i])) {
            (Some(high), Some(low)) => f32::from(high * 16 + low) * INV_255,
            _ => 0.0,
        };
        ((value * 255.0) as u8).max(48)
    })
}

const INV_255: f32 = 1.0 / 255.0;
