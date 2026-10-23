//! `WipeTower.cpp:522–659,1182–1233`: Type1 writer state and linear motion.
mod path;

use super::super::writer::Extrusion;
use std::fmt::Write;

pub(super) struct Writer {
    pub(super) gcode: String,
    pub(super) position: [f32; 2],
    pub(super) start: [f32; 2],
    pub(super) tool: usize,
    pub(super) used: f32,
    pub(super) extrusions: Vec<Extrusion>,
    pub(super) wipe_path: Vec<[f32; 2]>,
    pub(super) flow: f32,
    pub(super) max_e_speed: f32,
    pub(super) suppress_preview: bool,
    height: f32,
    area: f32,
    feedrate: f32,
    size: [f32; 2],
    angle: f32,
    y_shift: f32,
}

impl Writer {
    pub(super) fn new(height: f32, width: f32, area: f32, tool: usize) -> Self {
        Self {
            gcode: format!(
                "; LAYER_HEIGHT: {height:.6}\n; FEATURE: Prime tower\n; LINE_WIDTH: {width:.6}\n"
            ),
            position: [f32::MAX; 2],
            start: [0.0; 2],
            tool,
            used: 0.0,
            extrusions: Vec::new(),
            wipe_path: Vec::new(),
            flow: 0.0,
            max_e_speed: f32::MAX,
            suppress_preview: false,
            height,
            area,
            feedrate: 0.0,
            size: [0.0; 2],
            angle: 0.0,
            y_shift: 0.0,
        }
    }

    pub(super) fn set_position(&mut self, point: [f32; 2], size: [f32; 2], angle: f32) {
        self.size = size;
        self.angle = angle;
        self.start = self.rotated(point);
        self.position = point;
    }

    pub(super) fn set_y_shift(&mut self, shift: f32) {
        self.position[1] -= shift - self.y_shift;
        self.y_shift = shift;
    }

    pub(super) fn rotated(&self, point: [f32; 2]) -> [f32; 2] {
        let x = f64::from(point[0] - self.size[0] / 2.0);
        let y = f64::from(point[1] + (self.y_shift - self.size[1] / 2.0));
        let angle = f64::from(self.angle * (std::f64::consts::PI / 180.0) as f32);
        let (sin, cos) = angle.sin_cos();
        [
            (x * cos - y * sin) as f32 + self.size[0] / 2.0,
            (x * sin + y * cos) as f32 + self.size[1] / 2.0,
        ]
    }

    pub(super) fn explicit(
        &mut self,
        point: [f32; 2],
        amount: f32,
        mut speed: f32,
        (record, limit): (bool, bool),
    ) {
        if point == self.position && amount == 0.0 && (speed == 0.0 || speed == self.feedrate) {
            return;
        }
        let dx = point[0] - self.position[0];
        let dy = point[1] - self.position[1];
        let length = (dx * dx + dy * dy).sqrt();
        if record {
            self.used += amount;
        }
        let start = self.rotated(self.position);
        let end = self.rotated(point);
        if !self.suppress_preview && amount > 0.0 && length > 0.0 {
            let width = amount * self.area / (length * self.height)
                + self.height * (1.0 - std::f64::consts::FRAC_PI_4) as f32;
            if self
                .extrusions
                .last()
                .is_none_or(|item| item.position != start)
            {
                self.extrusions.push(Extrusion {
                    position: start,
                    width: 0.0,
                });
            }
            self.extrusions.push(Extrusion {
                position: end,
                width,
            });
        }
        self.gcode.push_str("G1");
        for (axis, before, after) in [('X', start[0], end[0]), ('Y', start[1], end[1])] {
            if (after - before).abs() > 1e-4_f32 {
                write!(self.gcode, " {axis}{after:.3}").unwrap();
            }
        }
        if amount != 0.0 {
            write!(self.gcode, " E{amount:.4}").unwrap();
        }
        let distance = if length == 0.0 { amount.abs() } else { length };
        if speed != 0.0 && speed != self.feedrate {
            if limit {
                let e_speed = amount / (distance / speed * 60.0);
                speed /= (e_speed / self.max_e_speed).max(1.0);
            }
            self.format_speed(speed);
        }
        // Type1 retains bare G1 after coordinate quantization; Type2 drops it.
        self.gcode.push('\n');
        self.position = point;
    }

    pub(super) fn travel_lines(
        &mut self,
        x: [f32; 2],
        (count, mut rightward): (i32, bool),
        (speed, step): (f32, f32),
    ) {
        for i in 0..count {
            self.travel([x[usize::from(rightward)], self.position[1]], speed);
            if i + 1 != count {
                self.travel([self.position[0], self.position[1] + step], 0.0);
                rightward = !rightward;
            }
        }
    }

    pub(super) fn travel(&mut self, point: [f32; 2], speed: f32) {
        self.explicit(point, 0.0, speed, (false, true));
    }

    pub(super) fn extrude(&mut self, point: [f32; 2], speed: f32) {
        let dx = point[0] - self.position[0];
        let dy = point[1] - self.position[1];
        self.explicit(
            point,
            (dx * dx + dy * dy).sqrt() * self.flow,
            speed,
            (true, true),
        );
    }

    fn format_speed(&mut self, speed: f32) {
        write!(self.gcode, " F{}", (speed + 0.5).floor() as i32).unwrap();
        self.feedrate = speed;
    }

    pub(super) fn feedrate(&mut self, speed: f32) {
        if speed != self.feedrate {
            self.gcode.push_str("G1");
            self.format_speed(speed);
            self.gcode.push('\n');
        }
    }

    pub(super) fn load(&mut self, amount: f32, speed: f32) {
        if amount == 0.0 && (speed == 0.0 || speed == self.feedrate) {
            return;
        }
        self.gcode.push_str("G1");
        if amount != 0.0 {
            write!(self.gcode, " E{amount:.4}").unwrap();
        }
        if speed != 0.0 && speed != self.feedrate {
            self.format_speed(speed);
        }
        self.gcode.push('\n');
    }

    pub(super) fn add_wipe(&mut self, point: [f32; 2]) {
        self.wipe_path.push(self.rotated(point));
    }

    pub(super) fn spiral_ironing(&mut self, area: f32, step: f32, speed: f32) {
        let center = self.position;
        let mut upper = center.map(|v| v + step);
        let mut lower = center.map(|v| v - step);
        let count = (area.sqrt() / step / 2.0).ceil() as i32;
        for _ in 0..count {
            self.travel([upper[0], self.position[1]], speed);
            self.travel([self.position[0], upper[1]], speed);
            self.travel([lower[0], self.position[1]], speed);
            self.travel([self.position[0], lower[1]], speed);
            upper = upper.map(|v| v + step);
            lower = lower.map(|v| v - step);
        }
    }

    pub(super) fn temperature(&mut self, temperature: i32, wait: bool) {
        writeln!(
            self.gcode,
            "M{} S{temperature}",
            if wait { 109 } else { 104 }
        )
        .unwrap();
    }
}
