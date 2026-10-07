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
        if !matches!(
            curr.move_type,
            EMoveType::Extrude | EMoveType::Travel | EMoveType::Wipe
        ) {
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
        if prev.move_type != curr.move_type
            && !matches!(
                prev.move_type,
                EMoveType::Extrude | EMoveType::Travel | EMoveType::Wipe
            )
        {
            prev.actual_feedrate = entry;
        }
        let prev = *prev;
        let mut insert = |t| {
            inserted.extend(actual_vertex(&prev, &curr, t, cruise_feedrate).map(|v| (id, v)));
        };
        let accelerate = f64::from(accelerate_until);
        if EPSILON < accelerate && accelerate < f64::from(distance) - EPSILON {
            insert(accelerate_until / distance);
        }
        let has_deceleration = f64::from(distance - decelerate_after) > EPSILON;
        if has_deceleration && f64::from(decelerate_after) > accelerate + EPSILON {
            insert(decelerate_after / distance);
        }
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

/// The vertex at fraction `t` of `prev → curr`, unless it coincides with
/// either end. Only a same-type predecessor interpolates the extrusion
/// attributes (`:509-520`).
fn actual_vertex(
    prev: &MoveVertex,
    curr: &MoveVertex,
    t: f32,
    actual_feedrate: f32,
) -> Option<MoveVertex> {
    let position = std::array::from_fn(|i| lerp(prev.position[i], curr.position[i], t));
    if norm(position, prev.position) <= EPSILON || norm(position, curr.position) <= EPSILON {
        return None;
    }
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
