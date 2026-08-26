use std::collections::BTreeSet;

use super::{feed_matches, parser::Travel};

const MAX_ARC_RADIUS_QUANTIZATION_DIFFERENCE_MM: f64 = 0.001_5;

pub(super) fn compare(layer: usize, expected: &[Travel], actual: &[Travel]) -> Result<(), String> {
    if expected.len() != actual.len() {
        return Err(format!(
            "layer {layer} travel geometry count differs: expected {}, actual {}",
            expected.len(),
            actual.len()
        ));
    }
    match_each(expected, actual, travel_shape_matches).map_err(|expected| {
        format!("layer {layer} travel geometry differs: no match for {expected:?}")
    })?;
    match_each(expected, actual, |expected, actual| {
        feed_matches(expected.feed, actual.feed)
    })
    .map_err(|expected| format!("layer {layer} travel feed differs: no match for {expected:?}"))?;
    let expected_acceleration = expected
        .iter()
        .map(|travel| travel.acceleration.as_str())
        .collect::<BTreeSet<_>>();
    let actual_acceleration = actual
        .iter()
        .map(|travel| travel.acceleration.as_str())
        .collect::<BTreeSet<_>>();
    if expected_acceleration != actual_acceleration {
        return Err(format!(
            "layer {layer} travel acceleration differs: expected {expected_acceleration:?}, \
             actual {actual_acceleration:?}"
        ));
    }
    Ok(())
}

fn match_each<'a>(
    expected: &'a [Travel],
    actual: &[Travel],
    matches: impl Fn(&Travel, &Travel) -> bool,
) -> Result<(), &'a Travel> {
    let mut matched = vec![false; actual.len()];
    for expected in expected {
        if let Some(index) = actual
            .iter()
            .enumerate()
            .position(|(index, actual)| !matched[index] && matches(expected, actual))
        {
            matched[index] = true;
        } else {
            return Err(expected);
        }
    }
    Ok(())
}

// Island order changes both ends of inter-island XY moves and may rotate the
// destination loop. Command, Z profile, arc radius/turns, feed multiset, and
// acceleration values are the complete scheduler-independent travel shape.

fn travel_shape_matches(expected: &Travel, actual: &Travel) -> bool {
    let expected = &expected.motion;
    let actual = &actual.motion;
    let z_matches = expected.start.z == actual.start.z && expected.end.z == actual.end.z;
    let arc_matches = !matches!(expected.command.as_str(), "G2" | "G3")
        || arc_radius_matches(&expected.arc_center, &actual.arc_center)
            && expected.turns == actual.turns;
    expected.command == actual.command && z_matches && arc_matches
}

fn arc_radius_matches(expected: &[Option<String>; 2], actual: &[Option<String>; 2]) -> bool {
    arc_radius(expected)
        .zip(arc_radius(actual))
        .is_some_and(|(expected, actual)| {
            (expected - actual).abs() <= MAX_ARC_RADIUS_QUANTIZATION_DIFFERENCE_MM
        })
}

fn arc_radius(center: &[Option<String>; 2]) -> Option<f64> {
    let component = |value: &Option<String>| match value {
        Some(value) => value.parse::<f64>().ok(),
        None => Some(0.0),
    };
    let i = component(&center[0])?;
    let j = component(&center[1])?;
    (i != 0.0 || j != 0.0).then(|| i.hypot(j))
}
