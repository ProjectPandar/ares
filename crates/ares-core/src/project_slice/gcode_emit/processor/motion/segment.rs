//! The single-segment block builder (`MotionState::segment_block`).

use super::super::motion_util::{norm, scale, word};
use super::{GCodeFlavor, MotionBlock, MotionKind, MotionState};

impl MotionState {
    pub(super) fn segment_block(&self, delta: [f64; 4]) -> Option<MotionBlock> {
        // `move_length` (`GCodeProcessor.cpp:3965-3968`): the squared sum
        // accumulates in double, stores once into `float sq_xyz_length`,
        // and the square root is the float sqrt of the stored sum; an
        // E-only move returns `float(|E|)`.
        let sq_xyz = (delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2]) as f32;
        let e_only = sq_xyz <= 0.0;
        let distance = if e_only {
            delta[3].abs() as f32 as f64
        } else {
            f64::from(sq_xyz.sqrt())
        };
        if distance <= f64::EPSILON {
            return None;
        }
        let has_xy = delta[0] != 0.0 || delta[1] != 0.0;
        let mut acceleration = if e_only {
            self.retract_acceleration
        } else if self.wiping || (delta[3] > 0.0 && has_xy) {
            self.acceleration
        } else {
            self.travel_acceleration
        };
        let mut speed = self.feedrate;
        for (axis, delta) in delta.iter().enumerate() {
            let ratio = (delta / distance).abs();
            if ratio == 0.0 {
                continue;
            }
            let max_feedrate = self.max_feedrate[axis];
            if max_feedrate > 0.0 {
                speed = speed.min(max_feedrate / ratio);
            }
            let max_acceleration = self.max_acceleration[axis];
            acceleration = acceleration.min(max_acceleration / ratio);
        }
        Some(MotionBlock {
            distance,
            speed,
            acceleration,
            centripetal_acceleration: self.acceleration.max(1.0),
            jerk: self.effective_jerk(),
            direction: scale(delta, 1.0 / distance),
            extrude_factor: self.extrude_factor,
            kind: if !self.wiping && e_only && delta[3] > 0.0 {
                MotionKind::Unretract
            } else {
                MotionKind::Regular
            },
            e_only,
        })
    }
}
