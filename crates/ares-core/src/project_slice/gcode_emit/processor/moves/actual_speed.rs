//! `TimeMachine::calculate_time` move times and actual-speed vertices
//! with their `actual_feedrate` (`GCodeProcessor.cpp:458-572`) inserted by `GCodeProcessor::
//! calculate_time` (`:5962-6008`), one planner flush at a time.

use crate::gcode_processor::{EMoveType, MoveVertex};
use crate::project_slice::gcode_emit::processor::motion::Trapezoid;

/// libslic3r `EPSILON` (`libslic3r.h:52`).
const EPSILON: f64 = 1e-4;

/// One timed block's detection result (`TimeMachine::ActualSpeedMove`).
struct Detected {
    move_id: usize,
    splits: usize,
    speed: f32,
}

/// The actual-speed moves one `calculate_time` call collects.
#[derive(Default)]
pub(in crate::project_slice::gcode_emit::processor) struct Pass {
    detected: Vec<Detected>,
    splits: Vec<MoveVertex>,
}

impl Pass {
    /// `:472-573` for a block whose `move_id` is `id`.
    pub(in crate::project_slice::gcode_emit::processor) fn block(
        &mut self,
        moves: &mut crate::gcode_processor::ChunkedMoves,
        id: usize,
        time: f64,
        trapezoid: &Trapezoid,
    ) {
        moves[id].time = time as f32;
        let curr = moves[id];
        if !is_timed(curr.move_type) {
            return;
        }
        let Trapezoid {
            distance,
            accelerate_until,
            decelerate_after,
            cruise_feedrate,
            entry,
            exit,
        } = *trapezoid;
        let prev = &mut moves[id - 1];
        if prev.move_type != curr.move_type && !is_timed(prev.move_type) {
            prev.actual_feedrate = entry;
        }
        let prev = *prev;
        let before = self.splits.len();
        self.splits.extend(
            split_fractions(trapezoid)
                .filter_map(|t| actual_vertex(&prev, &curr, t, cruise_feedrate)),
        );
        let has_deceleration = has_deceleration(trapezoid);
        // `Trapezoid::is_cruise_only` (`GCodeProcessor.hpp:444`).
        let is_cruise_only =
            f64::from(((decelerate_after - accelerate_until) - distance).abs()) < EPSILON;
        self.detected.push(Detected {
            move_id: id,
            splits: self.splits.len() - before,
            speed: if is_cruise_only || !has_deceleration {
                cruise_feedrate
            } else {
                exit
            },
        });
    }

    /// `:5962-6000`: inserts the split vertices in front of their block's
    /// vertex. Returns `id_map` (sorted by the old `move_id`) and the
    /// inserted count that renumber the queued blocks (`:6003-6008`).
    pub(in crate::project_slice::gcode_emit::processor) fn insert(
        &mut self,
        moves: &mut crate::gcode_processor::ChunkedMoves,
    ) -> (Vec<(usize, usize)>, usize) {
        for detected in &self.detected {
            moves[detected.move_id].actual_feedrate = detected.speed;
            if let Some(next) = moves.get_mut(detected.move_id + 1)
                && next.move_type == EMoveType::Seam
            {
                next.actual_feedrate = detected.speed;
            }
        }
        super::insert::insert(
            moves,
            self.detected
                .drain(..)
                .map(|item| (item.move_id, item.splits)),
            &mut self.splits,
        )
    }
}

/// Only these vertex types get a block time entry — the Normal-mode vertex
/// filter ahead of the `g1_times_cache` push (`:656-662`).
pub(in crate::project_slice::gcode_emit::processor) fn is_timed(kind: EMoveType) -> bool {
    matches!(
        kind,
        EMoveType::Extrude | EMoveType::Travel | EMoveType::Wipe
    )
}

fn has_deceleration(trapezoid: &Trapezoid) -> bool {
    f64::from(trapezoid.distance - trapezoid.decelerate_after) > EPSILON
}

/// The acceleration end and deceleration start, as fractions of the block.
pub(in crate::project_slice::gcode_emit::processor) fn split_fractions(
    trapezoid: &Trapezoid,
) -> impl Iterator<Item = f32> {
    let Trapezoid {
        distance,
        accelerate_until,
        decelerate_after,
        ..
    } = *trapezoid;
    let accelerate = f64::from(accelerate_until);
    let accelerating = EPSILON < accelerate && accelerate < f64::from(distance) - EPSILON;
    let decelerating =
        has_deceleration(trapezoid) && f64::from(decelerate_after) > accelerate + EPSILON;
    let at_accelerate = accelerating.then(|| accelerate_until / distance);
    let at_decelerate = decelerating.then(|| decelerate_after / distance);
    at_accelerate.into_iter().chain(at_decelerate)
}

/// The point at fraction `t` of `prev → curr`, unless it coincides with
/// either end.
pub(in crate::project_slice::gcode_emit::processor) fn split_position(
    prev: [f32; 3],
    curr: [f32; 3],
    t: f32,
) -> Option<[f32; 3]> {
    let position = std::array::from_fn(|i| lerp(prev[i], curr[i], t));
    (norm(position, prev) > EPSILON && norm(position, curr) > EPSILON).then_some(position)
}

/// The vertex at fraction `t` of `prev → curr`, unless it coincides with
/// either end. Only a same-type predecessor interpolates the extrusion
/// attributes (`:509-520`); `fan_speed` stays the current move's (2.5.0
/// `GCodeProcessor.cpp:688`, `:719`).
fn actual_vertex(
    prev: &MoveVertex,
    curr: &MoveVertex,
    t: f32,
    actual_feedrate: f32,
) -> Option<MoveVertex> {
    let position = split_position(prev.position, curr.position, t)?;
    let interpolate = prev.move_type == curr.move_type;
    let mix = |a: f32, b: f32| if interpolate { lerp(a, b, t) } else { b };
    Some(MoveVertex {
        position,
        delta_extruder: mix(prev.delta_extruder, curr.delta_extruder),
        width: mix(prev.width, curr.width),
        height: mix(prev.height, curr.height),
        temperature: mix(prev.temperature, curr.temperature),
        actual_feedrate,
        time: 0.0,
        internal_only: true,
        ..*curr
    })
}

/// `lerp` (`libslic3r.h:300`): `(1 - t) * a + t * b`.
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    (1.0 - t) * a + t * b
}

// Eigen `Vec3f::norm()` unrolls its reduction by halving (`Redux.h`
// `redux_novec_unroller`): `x² + (y² + z²)`.
fn norm(a: [f32; 3], b: [f32; 3]) -> f64 {
    let d: [f32; 3] = std::array::from_fn(|i| a[i] - b[i]);
    f64::from((d[0] * d[0] + (d[1] * d[1] + d[2] * d[2])).sqrt())
}
