use std::fmt::Debug;

#[path = "semantic/lifecycle.rs"]
mod lifecycle;
#[path = "semantic/model.rs"]
mod model;
#[path = "semantic/parser.rs"]
mod parser;
#[cfg(test)]
#[path = "semantic/tests.rs"]
mod tests;
#[path = "semantic/travel.rs"]
mod travel;

use parser::{Deposition, SemanticGcode};

const MAX_FEED_DIFFERENCE: f64 = 10.0;
const MAX_FEED_RELATIVE_DIFFERENCE: f64 = 0.01;
const MAX_TIME_DIFFERENCE_SECONDS: u64 = 5;
const MAX_FILAMENT_LENGTH_DIFFERENCE_MM: f64 = 0.05;

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
    if expected.layers.len() != actual.layers.len() {
        return Err(format!(
            "layer count differs: expected {}, actual {}",
            expected.layers.len(),
            actual.layers.len()
        ));
    }

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
        travel::compare(layer, &expected.travels, &actual.travels)?;
        exact_layer(
            layer,
            "control events",
            &expected.controls,
            &actual.controls,
        )?;
    }
    Ok(())
}

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

fn compare_deposition(
    layer: usize,
    expected: &[Deposition],
    actual: &[Deposition],
) -> Result<(), String> {
    if expected.len() != actual.len() {
        return Err(format!(
            "layer {layer} deposition count differs: expected {}, actual {}",
            expected.len(),
            actual.len()
        ));
    }
    for (index, (expected, actual)) in expected.iter().zip(actual).enumerate() {
        if expected.feature == actual.feature
            && expected.width == actual.width
            && expected.motion == actual.motion
            && expected.extrusion == actual.extrusion
            && expected.acceleration == actual.acceleration
            && expected.fans == actual.fans
            && feed_matches(expected.feed, actual.feed)
        {
            continue;
        }
        return Err(format!(
            "layer {layer} deposition {} differs: expected {expected:?}, actual {actual:?}",
            index + 1
        ));
    }
    Ok(())
}

const MAX_CROSS_TARGET_DEPOSITION_COORDINATE_DIFFERENCE_MM: f64 = 0.15;
const MAX_CROSS_TARGET_DEPOSITION_EXTRUSION_DIFFERENCE_MM: f64 = 0.005;
const MAX_CROSS_TARGET_SPLIT_EXTRUSION_MM: f64 = 0.0015;
const MAX_CROSS_TARGET_DEPOSITION_FEED_DIFFERENCE: f64 = 20.0;

fn compare_deposition_cross_target(
    layer: usize,
    expected: &[Deposition],
    actual: &[Deposition],
) -> Result<(), String> {
    let (expected, actual): (Vec<_>, Vec<_>) = if expected.len() == actual.len() {
        (expected.iter().collect(), actual.iter().collect())
    } else {
        (
            expected
                .iter()
                .filter(|deposition| !is_split_move(deposition))
                .collect(),
            actual
                .iter()
                .filter(|deposition| !is_split_move(deposition))
                .collect(),
        )
    };
    if expected.len() != actual.len() {
        return Err(format!(
            "layer {layer} deposition count differs across targets: expected {}, actual {}",
            expected.len(),
            actual.len()
        ));
    }
    for (index, (expected, actual)) in expected.iter().zip(&actual).enumerate() {
        if !deposition_matches(expected, actual) {
            return Err(format!(
                "layer {layer} deposition {} differs across targets: expected {expected:?}, \
                 actual {actual:?}",
                index + 1
            ));
        }
    }
    Ok(())
}

fn is_split_move(deposition: &Deposition) -> bool {
    deposition.extrusion.parse::<f64>().unwrap().abs() <= MAX_CROSS_TARGET_SPLIT_EXTRUSION_MM
}
fn deposition_matches(expected: &Deposition, actual: &Deposition) -> bool {
    expected.feature == actual.feature
        && expected.width == actual.width
        && expected.acceleration == actual.acceleration
        && expected.fans == actual.fans
        && cross_target_feed_matches(expected.feed, actual.feed)
        && numeric_matches(
            expected.extrusion.parse().unwrap(),
            actual.extrusion.parse().unwrap(),
            MAX_CROSS_TARGET_DEPOSITION_EXTRUSION_DIFFERENCE_MM,
        )
        && motion_matches(&expected.motion, &actual.motion)
}

fn cross_target_feed_matches(expected: f64, actual: f64) -> bool {
    let absolute = (expected - actual).abs();
    let relative = absolute / expected.abs().max(f64::EPSILON);
    absolute <= MAX_CROSS_TARGET_DEPOSITION_FEED_DIFFERENCE && relative <= 0.01
}

fn motion_matches(expected: &model::MotionRecord, actual: &model::MotionRecord) -> bool {
    expected.command == actual.command
        && position_matches(&expected.start, &actual.start)
        && position_matches(&expected.end, &actual.end)
        && expected.turns == actual.turns
}

fn position_matches(expected: &model::Position, actual: &model::Position) -> bool {
    numeric_matches(
        expected.x.parse().unwrap(),
        actual.x.parse().unwrap(),
        MAX_CROSS_TARGET_DEPOSITION_COORDINATE_DIFFERENCE_MM,
    ) && numeric_matches(
        expected.y.parse().unwrap(),
        actual.y.parse().unwrap(),
        MAX_CROSS_TARGET_DEPOSITION_COORDINATE_DIFFERENCE_MM,
    ) && expected.z == actual.z
}

fn numeric_matches(expected: f64, actual: f64, tolerance: f64) -> bool {
    (expected - actual).abs() <= tolerance
}

fn feed_matches(expected: f64, actual: f64) -> bool {
    let absolute = (expected - actual).abs();
    let relative = absolute / expected.abs().max(f64::EPSILON);
    absolute <= MAX_FEED_DIFFERENCE && relative <= MAX_FEED_RELATIVE_DIFFERENCE
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

fn exact<T: Debug + PartialEq>(name: &str, expected: &T, actual: &T) -> Result<(), String> {
    if expected == actual {
        Ok(())
    } else {
        Err(format!(
            "{name} differs: expected {expected:?}, actual {actual:?}"
        ))
    }
}

fn exact_layer<T: Debug + PartialEq>(
    layer: usize,
    name: &str,
    expected: &T,
    actual: &T,
) -> Result<(), String> {
    exact(&format!("layer {layer} {name}"), expected, actual)
}
