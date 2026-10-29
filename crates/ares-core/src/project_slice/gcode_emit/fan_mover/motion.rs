//! GCodeReader motion tracking for the FanMover pass.

use super::{FanMover, helpers::LineMotion};

impl FanMover {
    pub(super) fn process_g_command(&mut self, command: &str, motion: &mut LineMotion) -> f64 {
        let mut time = 0.0;
        let number: i32 = command[1..].parse().unwrap_or(-1);
        if number == 0 || number == 1 {
            time = self.linear_motion_time(motion);
        }
        time
    }

    fn linear_motion_time(&self, motion: &mut LineMotion) -> f64 {
        let mut time = 0.0;
        let (dx, dy, dz) = (
            motion.x.map_or(0.0, |value| value - self.position[0]),
            motion.y.map_or(0.0, |value| value - self.position[1]),
            motion.z.map_or(0.0, |value| value - self.position[2]),
        );
        motion.dx = dx;
        motion.dy = dy;
        motion.dz = dz;
        let (dx, dy, dz) = (f64::from(dx), f64::from(dy), f64::from(dz));
        let dist_sq = dx * dx + dy * dy + dz * dz;
        if dist_sq > 0.0 {
            let dist = dist_sq.sqrt();
            motion.dist = dist as f32;
            if self.current_speed > 0.0 {
                time = dist / self.current_speed;
            }
        }
        time
    }

    pub(super) fn track_motion(&mut self, motion: &mut LineMotion) {
        if let Some(x) = motion.x {
            self.position[0] = x;
        }
        if let Some(y) = motion.y {
            self.position[1] = y;
        }
        if let Some(z) = motion.z {
            self.position[2] = z;
        }
        if let Some(e) = motion.e {
            motion.de = if self.relative_e {
                e
            } else {
                e - self.e_position
            };
            self.e_position = if self.relative_e {
                self.e_position + e
            } else {
                e
            };
        }
    }
}
