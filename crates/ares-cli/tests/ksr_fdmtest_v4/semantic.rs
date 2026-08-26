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

pub(crate) fn compare(expected: &[u8], actual: &[u8]) -> Result<(), String> {
    let expected = parser::parse(expected)?;
    let actual = parser::parse(actual)?;

    compare_timing(&expected, &actual)?;
    compare_filament_lengths(&expected, &actual)?;
    exact("preamble", &expected.preamble, &actual.preamble)?;
    exact("postamble", &expected.postamble, &actual.postamble)?;
    if expected.layers.len() != actual.layers.len() {
        return Err(format!(
            "layer count differs: expected {}, actual {}",
            expected.layers.len(),
            actual.layers.len()
        ));
    }

    for (index, (expected, actual)) in expected.layers.iter().zip(&actual.layers).enumerate() {
        let layer = index + 1;
        exact_layer(layer, "metadata", &expected.metadata, &actual.metadata)?;
        compare_deposition(layer, &expected.deposition, &actual.deposition)?;
        lifecycle::compare(layer, &expected.lifecycles, &actual.lifecycles)?;
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

fn feed_matches(expected: f64, actual: f64) -> bool {
    let absolute = (expected - actual).abs();
    let relative = absolute / expected.abs().max(f64::EPSILON);
    absolute <= MAX_FEED_DIFFERENCE && relative <= MAX_FEED_RELATIVE_DIFFERENCE
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
