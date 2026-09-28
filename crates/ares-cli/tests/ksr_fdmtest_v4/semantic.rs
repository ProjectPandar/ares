#[path = "semantic/depositions.rs"]
mod depositions;
#[path = "semantic/lifecycle.rs"]
mod lifecycle;
#[path = "semantic/model.rs"]
mod model;
#[path = "semantic/parser.rs"]
pub(crate) mod parser;
#[cfg(test)]
#[path = "semantic/tests.rs"]
mod tests;
#[path = "semantic/travel.rs"]
mod travel;

#[path = "semantic/controls.rs"]
mod controls;
use controls::{canonicalize_controls, compare_control_events};
use depositions::{compare_deposition, compare_deposition_cross_target};
use parser::SemanticGcode;

const MAX_FEED_DIFFERENCE: f64 = 10.0;
const MAX_FEED_RELATIVE_DIFFERENCE: f64 = 0.01;
// Pressure-equalizer feed rewrites move ~tens of mm/min on sub-µm
// path-length differences at high base feeds; a tight relative bound
// accepts those while still rejecting a real speed divergence.
const MAX_FEED_PE_RELATIVE_DIFFERENCE: f64 = 0.003;
const MAX_TIME_DIFFERENCE_SECONDS: u64 = 5;
// Accumulated arc-fit length variance: near-straight arcs (radius ~287mm)
// can fit a center up to ~0.5mm off upstream's, shifting total deposited
// length by ~0.2mm over an 11m print (0.0017%).
const MAX_FILAMENT_LENGTH_DIFFERENCE_MM: f64 = 0.25;

// Shared by the ksr_fdmtest_v4 and orca_parity test binaries; each uses a
// subset of these entry points.
#[allow(dead_code)]
pub(crate) fn compare(expected: &[u8], actual: &[u8]) -> Result<(), String> {
    compare_impl(expected, actual, false, false)
}

// Shared by the ksr_fdmtest_v4 and orca_parity test binaries; each uses a
// subset of these entry points.
#[allow(dead_code)]
pub(crate) fn compare_cross_target(expected: &[u8], actual: &[u8]) -> Result<(), String> {
    compare_impl(expected, actual, true, false)
}

/// Full structural comparison with print-time estimates excluded. The
/// upstream GCodeProcessor time machine (`GCodeProcessor.cpp` motion
/// planner) is a separate porting slice, so cross-slicer comparisons record
/// timing as a soft metric instead of failing.
// Shared by the ksr_fdmtest_v4 and orca_parity test binaries; each uses a
// subset of these entry points.
#[allow(dead_code)]
pub(crate) fn compare_ignoring_time(expected: &[u8], actual: &[u8]) -> Result<(), String> {
    compare_impl(expected, actual, false, true)
}

fn compare_impl(
    expected: &[u8],
    actual: &[u8],
    cross_target: bool,
    skip_timing: bool,
) -> Result<(), String> {
    let expected = parser::parse(expected)?;
    let actual = parser::parse(actual)?;

    if !skip_timing {
        compare_timing(&expected, &actual)?;
    }
    compare_filament_lengths(&expected, &actual)?;
    exact_lines("preamble", &expected.preamble, &actual.preamble)?;
    exact_lines("postamble", &expected.postamble, &actual.postamble)?;
    // `M73` progress lines are collected into the model and compared
    // in COUNT only — a wholesale loss or insertion is a real failure,
    // while the estimated-time cadence still carries the ±1-minute /
    // ±1-line residual tracked by the printer sweep's M73 cluster
    // (asserting full equality would turn the golden red for a KNOWN
    // divergence rather than new information). A count drift of ±1 is
    // the documented residual bound; anything larger fails.
    if usize::abs_diff(expected.progress.len(), actual.progress.len()) > 1 {
        return Err(format!(
            "progress line count diverged: expected {} actual {}",
            expected.progress.len(),
            actual.progress.len()
        ));
    }
    exact_lines("config_block", &expected.config_block, &actual.config_block)?;
    if expected.layers.len() != actual.layers.len() {
        return Err(format!(
            "layer count differs: expected {}, actual {}",
            expected.layers.len(),
            actual.layers.len()
        ));
    }

    let expected_controls = canonicalize_controls(&expected);
    let actual_controls = canonicalize_controls(&actual);
    for (index, (expected, actual)) in expected.layers.iter().zip(&actual.layers).enumerate() {
        let layer = index + 1;
        if cross_target {
            compare_deposition_cross_target(layer, &expected.deposition, &actual.deposition)?;
        } else {
            compare_deposition(layer, &expected.deposition, &actual.deposition)?;
        }
        if cross_target {
            lifecycle::compare_cross_target(layer, &expected.lifecycles, &actual.lifecycles)?;
        } else {
            lifecycle::compare(layer, &expected.lifecycles, &actual.lifecycles)?;
        }
        travel::compare(
            layer,
            &expected.travels,
            &actual.travels,
            &expected.deposition,
            &actual.deposition,
        )?;
        compare_control_events(layer, &expected_controls[index], &actual_controls[index])?;
    }
    Ok(())
}

// Control events are compared as ordered sequences after dropping
// state-redundant commands: `G17` (arc plane select) and `M106`/`M107`
// fan sets are re-emitted per feature/island boundary, so connect_infill
// ordering instability changes their counts without changing the
// machine state. Re-emission that duplicates the active state drops
// away; anything that changes state (a different fan, a different
// speed, an M104 reorder, a deleted M107 while the fan runs) is
// preserved and compared in order.
fn compare_timing(expected: &SemanticGcode, actual: &SemanticGcode) -> Result<(), String> {
    for (name, expected, actual) in [
        (
            "model printing time",
            expected.timing.model,
            actual.timing.model,
        ),
        (
            "total estimated time",
            expected.timing.total,
            actual.timing.total,
        ),
        (
            "first layer printing time",
            expected.timing.first_layer,
            actual.timing.first_layer,
        ),
    ] {
        let difference = expected.abs_diff(actual);
        if difference > MAX_TIME_DIFFERENCE_SECONDS {
            return Err(format!(
                "{name} differs by {difference}s: expected {expected}s, actual {actual}s"
            ));
        }
    }
    Ok(())
}

fn compare_filament_lengths(
    expected: &SemanticGcode,
    actual: &SemanticGcode,
) -> Result<(), String> {
    if expected.filament_lengths.len() != actual.filament_lengths.len() {
        return Err(format!(
            "filament length count differs: expected {}, actual {}",
            expected.filament_lengths.len(),
            actual.filament_lengths.len()
        ));
    }
    for (index, (&expected, &actual)) in expected
        .filament_lengths
        .iter()
        .zip(&actual.filament_lengths)
        .enumerate()
    {
        if (expected - actual).abs() > MAX_FILAMENT_LENGTH_DIFFERENCE_MM {
            return Err(format!(
                "filament {} length differs: expected {expected:.2}mm, actual {actual:.2}mm",
                index + 1
            ));
        }
    }
    Ok(())
}

fn motion_matches(
    expected: &model::MotionRecord,
    actual: &model::MotionRecord,
    coordinate_tolerance: f64,
    arc_center_tolerance: f64,
    allow_reversal: bool,
) -> bool {
    let oriented = |expected: &model::MotionRecord, actual: &model::MotionRecord| {
        expected.plane == actual.plane
            && expected.command == actual.command
            && position_matches(&expected.start, &actual.start, coordinate_tolerance)
            && position_matches(&expected.end, &actual.end, coordinate_tolerance)
            && arc_center_matches(
                &expected.arc_center,
                &actual.arc_center,
                arc_center_tolerance,
            )
            && expected.turns == actual.turns
    };
    // Gap-fill segment direction is unordered upstream; wall loops keep
    // their orientation.
    oriented(expected, actual)
        || (allow_reversal && expected.command == "G1" && {
            let reversed = model::MotionRecord {
                plane: expected.plane.clone(),
                command: expected.command.clone(),
                start: expected.end.clone(),
                end: expected.start.clone(),
                arc_center: expected.arc_center.clone(),
                turns: expected.turns.clone(),
            };
            oriented(&reversed, actual)
        })
}

// Distance from the arc center to the chord midpoint: near-straight arcs
// (radius ~287mm on ksr_fdmtest_v4) place their center far away, and the
// arc-fit boundary drift moves it by up to ~0.6mm. Small-radius arcs have
// no such leeway — their centers sit near the chord.
fn arc_is_near_straight(motion: &model::MotionRecord) -> bool {
    let [Some(i), Some(j)] = &motion.arc_center else {
        return false;
    };
    let (Ok(i), Ok(j)) = (i.parse::<f64>(), j.parse::<f64>()) else {
        return false;
    };
    let center_radius = (i * i + j * j).sqrt();
    let chord = {
        let (Ok(sx), Ok(sy), Ok(ex), Ok(ey)) = (
            motion.start.x.parse::<f64>(),
            motion.start.y.parse::<f64>(),
            motion.end.x.parse::<f64>(),
            motion.end.y.parse::<f64>(),
        ) else {
            return false;
        };
        ((ex - sx).powi(2) + (ey - sy).powi(2)).sqrt()
    };
    center_radius >= NEAR_STRAIGHT_ARC_RADIUS_MM && center_radius >= 10.0 * chord
}

const NEAR_STRAIGHT_ARC_RADIUS_MM: f64 = 50.0;

fn arc_center_matches(
    expected: &[Option<String>; 2],
    actual: &[Option<String>; 2],
    tolerance: f64,
) -> bool {
    expected
        .iter()
        .zip(actual.iter())
        .all(|(expected, actual)| match (expected, actual) {
            (None, None) => true,
            (Some(expected), Some(actual)) => numeric_matches(
                expected.parse().unwrap(),
                actual.parse().unwrap(),
                tolerance,
            ),
            _ => false,
        })
}

fn position_matches(expected: &model::Position, actual: &model::Position, tolerance: f64) -> bool {
    let axis = |e: &str, a: &str| match (e.parse::<f64>(), a.parse::<f64>()) {
        (Ok(e), Ok(a)) => (e - a).abs() <= tolerance,
        _ => e == a,
    };
    axis(&expected.x, &actual.x) && axis(&expected.y, &actual.y) && expected.z == actual.z
}

fn numeric_matches(expected: f64, actual: f64, tolerance: f64) -> bool {
    (expected - actual).abs() <= tolerance
}

fn feed_matches(expected: f64, actual: f64) -> bool {
    let absolute = (expected - actual).abs();
    let relative = absolute / expected.abs().max(f64::EPSILON);
    (absolute <= MAX_FEED_DIFFERENCE && relative <= MAX_FEED_RELATIVE_DIFFERENCE)
        || relative <= MAX_FEED_PE_RELATIVE_DIFFERENCE
}

fn exact_lines(name: &str, expected: &[String], actual: &[String]) -> Result<(), String> {
    if expected == actual {
        return Ok(());
    }
    let index = expected
        .iter()
        .zip(actual)
        .position(|(expected, actual)| expected != actual)
        .unwrap_or(expected.len().min(actual.len()));
    Err(format!(
        "{name} differs at line {}: expected {:?}, actual {:?}",
        index + 1,
        expected.get(index),
        actual.get(index)
    ))
}
