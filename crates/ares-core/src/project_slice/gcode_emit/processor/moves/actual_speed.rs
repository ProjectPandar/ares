//! `TimeMachine::calculate_time` move times and actual-speed vertices
//! with their `actual_feedrate` (`GCodeProcessor.cpp:458-572`) inserted by `GCodeProcessor::
//! calculate_time` (`:5962-5997`).

use crate::gcode_processor::{EMoveType, MoveVertex};
use crate::project_slice::gcode_emit::processor::motion::Trapezoid;

/// libslic3r `EPSILON` (`libslic3r.h:52`).
const EPSILON: f64 = 1e-4;

pub(super) fn apply(
    moves: &mut Vec<MoveVertex>,
    block_moves: &[u32],
    times: &[f64],
    trapezoids: &[Trapezoid],
) {
    let mut inserted = Vec::new();
    let mut block_speeds = Vec::new();
    for ((&move_id, &time), trapezoid) in block_moves.iter().zip(times).zip(trapezoids) {
        let id = move_id as usize;
        moves[id].time = time as f32;
        let curr = moves[id];
        if !is_timed(curr.move_type) {
            continue;
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
        inserted.extend(
            split_fractions(trapezoid)
                .filter_map(|t| actual_vertex(&prev, &curr, t, cruise_feedrate))
                .map(|v| (id, v)),
        );
        let has_deceleration = has_deceleration(trapezoid);
        // `Trapezoid::is_cruise_only` (`GCodeProcessor.hpp:444`).
        let is_cruise_only =
            f64::from(((decelerate_after - accelerate_until) - distance).abs()) < EPSILON;
        block_speeds.push((
            id,
            if is_cruise_only || !has_deceleration {
                cruise_feedrate
            } else {
                exit
            },
        ));
    }
    // The insertion pass (`:5986-5997`) runs after every predecessor write and
    // only inserts before block moves, so a following Seam keeps index `id + 1`.
    for (id, speed) in block_speeds {
        moves[id].actual_feedrate = speed;
        if let Some(next) = moves.get_mut(id + 1)
            && next.move_type == EMoveType::Seam
        {
            next.actual_feedrate = speed;
        }
    }
    if inserted.is_empty() {
        return;
    }
    let mut merged = Vec::with_capacity(moves.len() + inserted.len());
    let mut inserted = inserted.into_iter().peekable();
    for (index, vertex) in moves.drain(..).enumerate() {
        while let Some((_, actual)) = inserted.next_if(|(base, _)| *base == index) {
            merged.push(actual);
        }
        merged.push(vertex);
    }
    *moves = merged;
}

/// Only these vertex types get a block time entry (`:484-488`).
pub(in crate::project_slice::gcode_emit::processor) fn is_timed(kind: EMoveType) -> bool {
    matches!(
        kind,
        EMoveType::Extrude | EMoveType::Travel | EMoveType::Wipe
    )
}

/// The number of actual-speed vertices `calculate_time` inserts in front of
/// a timed `curr` (`:497-546`).
pub(in crate::project_slice::gcode_emit::processor) fn split_count(
    prev: [f32; 3],
    curr: [f32; 3],
    trapezoid: &Trapezoid,
) -> usize {
    split_fractions(trapezoid)
        .filter(|&t| split_position(prev, curr, t).is_some())
        .count()
}

fn has_deceleration(trapezoid: &Trapezoid) -> bool {
    f64::from(trapezoid.distance - trapezoid.decelerate_after) > EPSILON
}

/// The acceleration end and deceleration start, as fractions of the block.
fn split_fractions(trapezoid: &Trapezoid) -> impl Iterator<Item = f32> {
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
fn split_position(prev: [f32; 3], curr: [f32; 3], t: f32) -> Option<[f32; 3]> {
    let position = std::array::from_fn(|i| lerp(prev[i], curr[i], t));
    (norm(position, prev) > EPSILON && norm(position, curr) > EPSILON).then_some(position)
}

/// The vertex at fraction `t` of `prev → curr`, unless it coincides with
/// either end. Only a same-type predecessor interpolates the extrusion
/// attributes (`:509-520`).
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
        fan_speed: mix(prev.fan_speed, curr.fan_speed),
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
