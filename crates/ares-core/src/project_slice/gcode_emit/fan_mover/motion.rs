//! GCodeReader motion tracking for the FanMover pass.

use super::{FanMover, helpers::LineMotion};

impl FanMover {
    pub(super) fn process_g_command(&mut self, command: &str, motion: &mut LineMotion) -> f32 {
        let mut time = 0.0;
        let number: i32 = command[1..].parse().unwrap_or(-1);
        if number == 0 || number == 1 {
            time = self.linear_motion_time(motion);
        } else if number == 90 {
            self.relative_xyz = false;
        } else if number == 91 {
            self.relative_xyz = true;
        }
        time
    }

    fn linear_motion_time(&self, motion: &mut LineMotion) -> f32 {
        let mut time = 0.0;
        let (dx, dy, dz) = (
            motion.x.map_or(0.0, |value| {
                if self.relative_xyz {
                    value
                } else {
                    value - self.position[0]
                }
            }),
            motion.y.map_or(0.0, |value| {
                if self.relative_xyz {
                    value
                } else {
                    value - self.position[1]
                }
            }),
            motion.z.map_or(0.0, |value| {
                if self.relative_xyz {
                    value
                } else {
                    value - self.position[2]
                }
            }),
        );
        motion.dx = dx;
        motion.dy = dy;
        motion.dz = dz;
        let dist_sq = dx * dx + dy * dy + dz * dz;
        if dist_sq > 0.0 {
            motion.dist = dist_sq.sqrt();
            if self.current_speed > 0.0 {
                time = motion.dist / self.current_speed;
            }
        }
        time
    }

    pub(super) fn track_motion(&mut self, motion: &mut LineMotion) {
        if let Some(x) = motion.x {
            self.position[0] = if self.relative_xyz {
                self.position[0] + x
            } else {
                x
            };
        }
        if let Some(y) = motion.y {
            self.position[1] = if self.relative_xyz {
                self.position[1] + y
            } else {
                y
            };
        }
        if let Some(z) = motion.z {
            self.position[2] = if self.relative_xyz {
                self.position[2] + z
            } else {
                z
            };
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
