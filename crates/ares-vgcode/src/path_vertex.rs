// Ports rendering-neutral data from AGPL-licensed OrcaSlicer `src/libvgcode/include/PathVertex.hpp` and `src/PathVertex.cpp`.

use crate::{GCodeExtrusionRole, MoveType, TimeMode, Vec3};

#[derive(Clone, Debug, PartialEq)]
pub struct PathVertex {
    pub position: Vec3,
    pub height: f32,
    pub width: f32,
    pub feedrate: f32,
    pub actual_feedrate: f32,
    pub mm3_per_mm: f32,
    pub fan_speed: f32,
    pub temperature: f32,
    pub role: GCodeExtrusionRole,
    pub move_type: MoveType,
    pub gcode_id: u32,
    pub layer_id: u32,
    pub extruder_id: u8,
    pub color_id: u8,
    pub times: [f32; TimeMode::COUNT],
    pub layer_duration: f32,
    pub pressure_advance: f32,
    pub acceleration: f32,
    pub jerk: f32,
}

impl PathVertex {
    pub const DUMMY_PATH_VERTEX: Self = Self {
        position: [f32::MAX, f32::MAX, f32::MAX],
        height: 0.0,
        width: 0.0,
        feedrate: 0.0,
        actual_feedrate: 0.0,
        mm3_per_mm: 0.0,
        fan_speed: 0.0,
        temperature: 0.0,
        role: GCodeExtrusionRole::None,
        move_type: MoveType::Noop,
        gcode_id: 0,
        layer_id: 0,
        extruder_id: 0,
        color_id: 0,
        times: [0.0; TimeMode::COUNT],
        layer_duration: 0.0,
        pressure_advance: 0.0,
        acceleration: 0.0,
        jerk: 0.0,
    };

    pub fn is_extrusion(&self) -> bool {
        self.move_type == MoveType::Extrude
    }

    pub fn is_travel(&self) -> bool {
        self.move_type == MoveType::Travel
    }

    pub fn is_wipe(&self) -> bool {
        self.move_type == MoveType::Wipe
    }

    pub fn is_option(&self) -> bool {
        matches!(
            self.move_type,
            MoveType::Retract
                | MoveType::Unretract
                | MoveType::Seam
                | MoveType::ToolChange
                | MoveType::ColorChange
                | MoveType::PausePrint
                | MoveType::CustomGCode
        )
    }

    pub fn is_custom_gcode(&self) -> bool {
        self.move_type == MoveType::Extrude && self.role == GCodeExtrusionRole::Custom
    }

    pub fn volumetric_rate(&self) -> f32 {
        self.feedrate * self.mm3_per_mm
    }

    pub fn actual_volumetric_rate(&self) -> f32 {
        self.actual_feedrate * self.mm3_per_mm
    }
}

impl Default for PathVertex {
    fn default() -> Self {
        Self::DUMMY_PATH_VERTEX
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_matches_upstream_path_vertex() {
        let vertex = PathVertex::default();
        assert_eq!(vertex.position, [f32::MAX, f32::MAX, f32::MAX]);
        assert_eq!(vertex.role, GCodeExtrusionRole::None);
        assert_eq!(vertex.move_type, MoveType::Noop);
        assert_eq!(vertex.times, [0.0, 0.0]);
        assert_eq!(vertex, PathVertex::DUMMY_PATH_VERTEX);
    }

    #[test]
    fn helper_methods_match_move_and_role_semantics() {
        let mut vertex = PathVertex {
            move_type: MoveType::Extrude,
            ..PathVertex::default()
        };
        assert!(vertex.is_extrusion());
        assert!(!vertex.is_option());
        assert!(!vertex.is_custom_gcode());

        vertex.role = GCodeExtrusionRole::Custom;
        assert!(vertex.is_custom_gcode());

        vertex.move_type = MoveType::Travel;
        assert!(vertex.is_travel());
        assert!(!vertex.is_custom_gcode());

        vertex.move_type = MoveType::Wipe;
        assert!(vertex.is_wipe());

        vertex.move_type = MoveType::PausePrint;
        assert!(vertex.is_option());
    }

    #[test]
    fn volumetric_rates_use_feedrates_and_mm3_per_mm() {
        let vertex = PathVertex {
            feedrate: 10.0,
            actual_feedrate: 8.0,
            mm3_per_mm: 0.25,
            ..PathVertex::default()
        };
        assert_eq!(vertex.volumetric_rate(), 2.5);
        assert_eq!(vertex.actual_volumetric_rate(), 2.0);
    }
}
