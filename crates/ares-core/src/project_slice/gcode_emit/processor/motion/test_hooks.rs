//! Test-only planner exposure (`MotionState` tests).

use super::MotionBlock;
use super::planner;

#[cfg(test)]
pub(in crate::project_slice) fn planned_times(blocks: &[MotionBlock]) -> Vec<f64> {
    planner::planned_times(blocks)
}

#[cfg(test)]
pub(in crate::project_slice) fn planned_trapezoid_time(
    distance: f32,
    cruise: f32,
    entry: f32,
    exit: f32,
    acceleration: f32,
) -> f32 {
    planner::planned_trapezoid_time(distance, cruise, entry, exit, acceleration)
}
