//! The single-segment block builder (`MotionState::segment_block`).

use super::super::motion_util::{norm, scale, word};
use super::{GCodeFlavor, MotionBlock, MotionKind, MotionState};

impl MotionState {
    pub(super) fn segment_block(&self, delta: [f64; 4]) -> Option<MotionBlock> {
        let xyz_distance = norm([delta[0], delta[1], delta[2], 0.0]);
        let e_only = xyz_distance <= f64::EPSILON;
        let distance = if e_only { delta[3].abs() } else { xyz_distance };
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
