use ares_core::{
    ExtrusionRole, ProcessedGCode,
    gcode_processor::{EMoveType, GCodeProcessorResult, MoveVertex},
};

pub fn input(large: bool) -> ProcessedGCode {
    let count = if large { 1_048_579 } else { 37 };
    let moves = std::iter::once(MoveVertex::default())
        .chain((0..count).map(|index| MoveVertex {
            gcode_id: index + 7,
            move_type: if index % 5 == 0 {
                EMoveType::Travel
            } else {
                EMoveType::Extrude
            },
            extrusion_role: if index % 2 == 0 {
                ExtrusionRole::Perimeter
            } else {
                ExtrusionRole::ExternalPerimeter
            },
            extruder_id: (index % 2) as u8,
            cp_color_id: (index % 2) as u8,
            position: [
                (index % 31) as f32,
                (index % 17) as f32,
                0.2 * (index % 3 + 1) as f32,
            ],
            delta_extruder: 0.1,
            feedrate: 30.0 + (index % 11) as f32,
            width: 0.4,
            height: 0.2,
            mm3_per_mm: 0.08,
            travel_dist: 1.0,
            fan_speed: 50.0,
            temperature: 210.0,
            pressure_advance: 0.02,
            acceleration: 500.0,
            jerk: 8.0,
            actual_feedrate: 20.0 + (index % 7) as f32,
            time: 0.125,
            layer_id: index % 3,
            internal_only: index % 13 == 0,
        }))
        .collect();
    ProcessedGCode {
        gcode: Vec::new(),
        warnings: Vec::new(),
        result: GCodeProcessorResult {
            moves,
            extruder_colors: vec!["#FF0000".into(), "#00FF00".into()],
            ..Default::default()
        },
        print_statistics: Default::default(),
    }
}

pub const STATES: [(&str, u32, u16, i32); 4] = [
    ("FeatureType", u32::MAX, u16::MAX, 3),
    ("Speed", u32::MAX, 0, 5),
    ("ActualSpeed", 1 << 1, u16::MAX, 2),
    ("Height", u32::MAX, u16::MAX, 4),
];
