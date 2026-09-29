//! Junction-speed and acceleration-limited velocity computation
//! (`GCodeProcessor.cpp:4085-4182, 149-152`).

use super::{JunctionInput, PlannedBlock};

/// Upstream's junction vectors are `Vec3f` (`GCodeProcessor.cpp:4091-
/// 4097`): `exit_v = prev.feedrate * prev.exit_direction` multiplies
/// f32×f32 with a single rounding per component. Computing the exit
/// through the f64 direction keeps a different last ulp that flips the
/// following axis' `> jerk` comparison (12.000001 vs 11.999999) and
/// rescales the jerk vector twice — inflating the junction speed
/// (3.5355 vs the oracle's 2.0 on the Ender-3 V3 wipe→travel
/// junction).
pub(super) fn junction_speed(previous: PlannedBlock, current: JunctionInput) -> f32 {
    let JunctionInput {
        direction,
        jerk,
        cruise,
        axis_feedrate,
        safe,
    } = current;
    if previous.cruise <= 0.000_1 {
        return safe;
    }
    let previous_larger = previous.cruise > cruise;
    let smaller_factor = if previous_larger {
        cruise / previous.cruise
    } else {
        previous.cruise / cruise
    };
    let mut maximum = previous.cruise.min(cruise);
    let mut factor = 1.0;
    let mut limited = false;

    let mut exit = previous
        .direction
        .map(|direction| previous.cruise * direction);
    if previous_larger {
        exit.iter_mut().for_each(|value| *value *= smaller_factor);
    }
    // Upstream computes the junction jerk vector in `Vec3f`
    // (`GCodeProcessor.cpp:4091-4097`: `Vec3f exit_v/entry_v/jerk_v`),
    // so every product rounds to f32 BEFORE the comparisons. An f64
    // difference keeps a ~1e-6 residue above the jerk limit that re-fires
    // the following axis check, rescales the remaining deltas, and
    // inflates the junction speed (the wipe→travel junction 3.5355 vs
    // the oracle's 2.0).
    let mut difference = [
        (cruise * direction[0] - exit[0]).abs(),
        (cruise * direction[1] - exit[1]).abs(),
        (cruise * direction[2] - exit[2]).abs(),
    ];
    for axis in 0..3 {
        if difference[axis] > jerk[axis] {
            factor *= jerk[axis] / difference[axis];
            difference.iter_mut().for_each(|value| *value *= factor);
            limited = true;
        }
    }

    // `GCodeProcessor.cpp:4123-4131`: `v_exit`/`v_entry` are `float`
    // narrowed BEFORE the factor multiplications; each product rounds
    // to f32 (upstream `float` fields in `axis_feedrate` are already
    // f32 — keep e_exit/e_entry f32 end-to-end).
    let mut e_exit = previous.axis_feedrate[3] as f32;
    let mut e_entry = axis_feedrate[3] as f32;
    if previous_larger {
        e_exit *= smaller_factor;
    }
    if limited {
        e_exit *= factor;
        e_entry *= factor;
    }
    let e_jerk = axis_jerk(e_exit, e_entry);
    if e_jerk > jerk[3] {
        factor *= jerk[3] / e_jerk;
        limited = true;
    }
    if limited {
        maximum *= factor;
    }
    if previous.safe > maximum * 0.99 && safe > maximum * 0.99 {
        safe
    } else {
        maximum
    }
}

/// `GCodeProcessor.cpp:149-152`.
pub(super) fn max_allowable_speed(acceleration: f32, target: f32, distance: f32) -> f32 {
    (target * target + 2.0 * acceleration * distance).sqrt()
}

fn axis_jerk(exit: f32, entry: f32) -> f32 {
    if exit > entry {
        if entry > 0.0 || exit < 0.0 {
            exit - entry
        } else {
            exit.max(-entry)
        }
    } else if entry < 0.0 || exit > 0.0 {
        entry - exit
    } else {
        (-exit).max(entry)
    }
}
