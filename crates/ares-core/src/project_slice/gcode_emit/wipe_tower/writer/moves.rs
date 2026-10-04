use super::{Extrusion, Writer};
use std::fmt::Write;

#[derive(Clone, Copy)]
pub(in super::super) struct Extrude {
    pub(in super::super) amount: f32,
    pub(in super::super) feedrate: f32,
    pub(in super::super) record: bool,
    pub(in super::super) limit: bool,
}

impl Writer {
    pub(in super::super) fn travel(&mut self, point: [f32; 2], feedrate: f32) {
        self.extrude_explicit(
            point,
            Extrude {
                amount: 0.0,
                feedrate,
                record: false,
                limit: true,
            },
        );
    }

    pub(in super::super) fn extrude(&mut self, point: [f32; 2], feedrate: f32) {
        let dx = point[0] - self.position[0];
        let dy = point[1] - self.position[1];
        self.extrude_explicit(
            point,
            Extrude {
                amount: (dx * dx + dy * dy).sqrt() * self.flow,
                feedrate,
                record: true,
                limit: true,
            },
        );
    }

    pub(in super::super) fn extrude_explicit(&mut self, point: [f32; 2], movement: Extrude) {
        let Extrude {
            amount,
            mut feedrate,
            record,
            limit,
        } = movement;
        if point == self.position && amount == 0.0 && (feedrate == 0.0 || feedrate == self.feedrate)
        {
            return;
        }
        let dx = point[0] - self.position[0];
        let dy = point[1] - self.position[1];
        let length = (dx * dx + dy * dy).sqrt();
        if record {
            self.used_filament += amount;
        }
        let start = self.rotated(self.position);
        let end = self.rotated(point);
        if !self.suppress_preview && amount > 0.0 && length > 0.0 {
            // Upstream deliberately uses the first filament's area for preview.
            let width = amount * self.first_filament_area / (length * self.layer_height)
                + self.layer_height * (1.0 - std::f64::consts::FRAC_PI_4) as f32;
            if self.extrusions.last().is_none_or(|e| e.position != start) {
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
        let begin = self.gcode.len();
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
        if feedrate != 0.0 && feedrate != self.feedrate {
            if limit {
                let e_speed = amount / (distance / feedrate * 60.0);
                feedrate /= (e_speed / self.max_e_speed).max(1.0);
            }
            self.format_feedrate(feedrate);
        }
        if self.gcode.len() == begin + 2 {
            self.gcode.truncate(begin);
        } else {
            self.gcode.push('\n');
        }
        self.position = point;
    }

    pub(in super::super) fn rectangle(&mut self, lower: [f32; 2], size: [f32; 2], feedrate: f32) {
        let upper = [lower[0] + size[0], lower[1] + size[1]];
        let corners = [lower, [upper[0], lower[1]], upper, [lower[0], upper[1]]];
        let right = self.position[0] - lower[0] > upper[0] - self.position[0];
        let top = self.position[1] - lower[1] > upper[1] - self.position[1];
        let closest = match (right, top) {
            (false, false) => 0,
            (true, false) => 1,
            (true, true) => 2,
            (false, true) => 3,
        };
        self.travel([corners[closest][0], self.position[1]], 0.0);
        self.travel(corners[closest], 0.0);
        for offset in 1..=4 {
            self.extrude(corners[(closest + offset) % 4], feedrate);
        }
    }

    pub(in super::super) fn load_move_x(
        &mut self,
        farthest: f32,
        amount: f32,
        speed: f32,
        max_x_speed: f32,
    ) {
        let time = (amount / speed).abs();
        let mut distance = (farthest - self.position[0]).abs();
        let mut x_speed = distance / time;
        if x_speed > max_x_speed {
            distance = max_x_speed * time;
            x_speed = max_x_speed;
        }
        let sign = if farthest > self.position[0] {
            1.0
        } else {
            -1.0
        };
        self.extrude_explicit(
            [self.position[0] + sign * distance, self.position[1]],
            Extrude {
                amount,
                feedrate: x_speed * 60.0,
                record: false,
                limit: false,
            },
        );
    }

    pub(in super::super) fn load_move_x_there_and_back(
        &mut self,
        farthest: f32,
        amount: f32,
        speed: f32,
        x_speed: f32,
    ) {
        let start = self.position;
        let time = (amount / speed).abs();
        let max_distance = (farthest - start[0]).abs();
        let distance = x_speed * time;
        let count = (distance / (2.0 * max_distance) + 1.0) as i32;
        let ratio = 2.0 * count as f32 * max_distance / distance;
        let sign = if farthest > start[0] { 1.0 } else { -1.0 };
        let end = [start[0] + sign * max_distance / ratio, start[1]];
        let movement = Extrude {
            amount: amount / (2.0 * count as f32),
            feedrate: x_speed * 60.0,
            record: false,
            limit: false,
        };
        for _ in 0..count {
            self.extrude_explicit(end, movement);
            self.extrude_explicit(start, movement);
        }
    }

    pub(in super::super) fn spiral_flat_ironing(
        &mut self,
        center: [f32; 2],
        area: f32,
        step: f32,
        feedrate: f32,
    ) {
        let mut upper = [center[0] + step, center[1] + step];
        let mut lower = [center[0] - step, center[1] - step];
        let count = (area.sqrt() / step / 2.0).ceil() as i32;
        for _ in 0..count {
            self.travel([upper[0], self.position[1]], feedrate);
            self.travel([self.position[0], upper[1]], feedrate);
            self.travel([lower[0], self.position[1]], feedrate);
            self.travel([self.position[0], lower[1]], feedrate);
            upper = upper.map(|v| v + step);
            lower = lower.map(|v| v - step);
        }
    }
}
