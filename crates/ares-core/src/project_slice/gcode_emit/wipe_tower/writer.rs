//! `WipeTower2.cpp:561–1218`: private non-BBL tower writer.

mod commands;
mod moves;
mod path;
pub(super) use moves::Extrude;

use crate::GCodeFlavor;

#[derive(Clone)]
pub(super) struct Extrusion {
    pub(super) position: [f32; 2],
    pub(super) width: f32,
}

pub(super) struct Writer {
    pub(super) gcode: String,
    pub(super) position: [f32; 2],
    pub(super) start: [f32; 2],
    pub(super) tool: usize,
    pub(super) used_filament: f32,
    pub(super) extrusions: Vec<Extrusion>,
    pub(super) wipe_path: Vec<[f32; 2]>,
    pub(super) suppress_preview: bool,
    feedrate: f32,
    flow: f32,
    max_e_speed: f32,
    first_filament_area: f32,
    layer_height: f32,
    size: [f32; 2],
    angle: f32,
    y_shift: f32,
    flavor: GCodeFlavor,
}

impl Writer {
    pub(super) fn new(
        height: f32,
        width: f32,
        flavor: GCodeFlavor,
        first_filament_area: f32,
    ) -> Self {
        Self {
            gcode: format!(
                ";HEIGHT:{}\n;TYPE:Prime tower\n;WIDTH:{}\n",
                super::super::format_processor_float(f64::from(height)),
                super::super::format_processor_float(f64::from(width))
            ),
            position: [f32::MAX; 2],
            start: [0.0; 2],
            tool: 0,
            used_filament: 0.0,
            extrusions: Vec::new(),
            wipe_path: Vec::new(),
            suppress_preview: false,
            feedrate: 0.0,
            flow: 0.0,
            max_e_speed: f32::MAX,
            first_filament_area,
            layer_height: height,
            size: [0.0; 2],
            angle: 0.0,
            y_shift: 0.0,
            flavor,
        }
    }

    pub(super) fn set_initial_position(&mut self, position: [f32; 2], size: [f32; 2], angle: f32) {
        self.size = size;
        self.angle = angle;
        self.start = self.rotated(position);
        self.position = position;
    }

    pub(super) fn set_tool(&mut self, tool: usize, max_e_speed: f32) {
        self.tool = tool;
        self.max_e_speed = max_e_speed;
    }

    pub(super) fn change_width(&mut self, width: f32) {
        self.gcode.push_str(&format!(
            ";WIDTH:{}\n",
            super::super::format_processor_float(f64::from(width))
        ));
    }

    pub(super) fn set_flow(&mut self, flow: f32) {
        self.flow = flow;
    }

    pub(super) fn set_y_shift(&mut self, shift: f32) {
        self.position[1] -= shift - self.y_shift;
        self.y_shift = shift;
    }

    pub(super) fn rotated(&self, position: [f32; 2]) -> [f32; 2] {
        let x = f64::from(position[0] - self.size[0] / 2.0);
        let y = f64::from(position[1] + (self.y_shift - self.size[1] / 2.0));
        let angle = f64::from(self.angle * (std::f64::consts::PI / 180.0) as f32);
        let (sin, cos) = angle.sin_cos();
        [
            (x * cos - y * sin) as f32 + self.size[0] / 2.0,
            (x * sin + y * cos) as f32 + self.size[1] / 2.0,
        ]
    }

    pub(super) fn add_wipe_point(&mut self, point: [f32; 2]) {
        self.wipe_path.push(self.rotated(point));
    }

    pub(super) fn take_used_filament(&mut self) -> f32 {
        std::mem::take(&mut self.used_filament)
    }
}
