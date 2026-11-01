// Ports AGPL-licensed OrcaSlicer `src/slic3r/GUI/LibVGCode/LibVGCodeWrapper.cpp`
// `convert(const GCodeProcessorResult&, ...)` (`:191-268`) and the role / move
// type / color conversions it uses (`:46-146`).

use ares_core::ExtrusionRole;
use ares_core::gcode_processor::{CustomGCodeType, EMoveType, GCodeProcessorResult, MoveVertex};

use crate::{
    Color, DUMMY_COLOR, GCodeExtrusionRole, GCodeInputData, MoveType, OptionType, PathVertex,
    TimeMode, move_type_to_option,
};

/// `LibVGCodeWrapper::convert` with result-derived palettes from
/// `Plater.cpp:16927-16946` and `GUI_Preview.cpp:680-688` (G-code viewer path).
pub fn from_processor_result(result: &GCodeProcessorResult) -> GCodeInputData {
    let tools_colors: Vec<_> = result
        .extruder_colors
        .iter()
        .map(|color| decode(color))
        .collect();
    let mut color_print_colors = tools_colors.clone();
    if !result.custom_gcode_per_print_z.is_empty() {
        color_print_colors.extend(
            result
                .custom_gcode_per_print_z
                .iter()
                .filter(|item| item.kind == CustomGCodeType::ColorChange)
                .map(|item| decode(&item.color)),
        );
        color_print_colors.push([128; 3]);
    }
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
            || prev.mm3_per_mm != curr.mm3_per_mm
            // ORCA: split the path when a preview value changes
            // (LibVGCodeWrapper.cpp:236-238).
            || prev.acceleration != curr.acceleration
            || prev.jerk != curr.jerk);
        if starts_path {
            // Phantom vertex at the previous position so libvgcode detects the
            // path start; its times stay zero.
            vertices.push(PathVertex {
                position: prev.position,
                actual_feedrate: prev.actual_feedrate,
                times: [0.0; TimeMode::COUNT],
                ..vertex(curr, move_type)
            });
        }
        vertices.push(vertex(curr, move_type));
    }
    GCodeInputData {
        spiral_vase_mode: result.spiral_vase_mode,
        vertices,
        tools_colors,
        color_print_colors,
    }
}

/// Stealth time (deferred by M853) stays zero.
fn vertex(curr: &MoveVertex, move_type: MoveType) -> PathVertex {
    PathVertex {
        position: curr.position,
        height: curr.height,
        width: curr.width,
        feedrate: curr.feedrate,
        actual_feedrate: curr.actual_feedrate,
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
        pressure_advance: curr.pressure_advance,
        acceleration: curr.acceleration,
        jerk: curr.jerk,
    }
}

/// GCodeViewer::load_as_preview `m_move_type_counts` (`GCodeViewer.cpp:1318-1324`):
/// processed moves by `MoveType` index, internal split moves skipped.
pub fn move_type_counts(moves: &[MoveVertex]) -> [u32; MoveType::COUNT] {
    let mut counts = [0; MoveType::COUNT];
    for m in moves.iter().filter(|m| !m.internal_only) {
        counts[move_type(m.move_type).index()] += 1;
    }
    counts
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
