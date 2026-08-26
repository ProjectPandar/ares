use super::{exact_layer, model::LifecycleEvent};

pub(super) fn compare(
    layer: usize,
    expected: &[Vec<LifecycleEvent>],
    actual: &[Vec<LifecycleEvent>],
) -> Result<(), String> {
    compare_with_tolerance(layer, expected, actual, false)
}

pub(super) fn compare_cross_target(
    layer: usize,
    expected: &[Vec<LifecycleEvent>],
    actual: &[Vec<LifecycleEvent>],
) -> Result<(), String> {
    compare_with_tolerance(layer, expected, actual, true)
}

fn compare_with_tolerance(
    layer: usize,
    expected: &[Vec<LifecycleEvent>],
    actual: &[Vec<LifecycleEvent>],
    tolerate_float_drift: bool,
) -> Result<(), String> {
    if !tolerate_float_drift {
        return exact_layer(layer, "island lifecycle", &expected, &actual);
    }
    if expected.len() != actual.len() {
        return Err(format!(
            "layer {layer} island lifecycle differs: expected {expected:?}, actual {actual:?}"
        ));
    }
    for (expected, actual) in expected.iter().zip(actual) {
        if expected.len() != actual.len()
            || expected
                .iter()
                .zip(actual)
                .any(|(expected, actual)| !event_matches(expected, actual))
        {
            return Err(format!(
                "layer {layer} island lifecycle differs: expected {expected:?}, actual {actual:?}"
            ));
        }
    }
    Ok(())
}

fn event_matches(expected: &LifecycleEvent, actual: &LifecycleEvent) -> bool {
    match (expected, actual) {
        (
            LifecycleEvent::Extruder {
                extrusion: expected_extrusion,
                feed: expected_feed,
            },
            LifecycleEvent::Extruder {
                extrusion: actual_extrusion,
                feed: actual_feed,
            },
        ) => {
            numeric_matches(expected_extrusion, actual_extrusion, 0.000011)
                && expected_feed == actual_feed
        }
        (LifecycleEvent::WipeStart, LifecycleEvent::WipeStart)
        | (LifecycleEvent::WipeEnd, LifecycleEvent::WipeEnd) => true,
        (
            LifecycleEvent::Wipe {
                motion: expected_motion,
                extrusion: expected_extrusion,
                feed: expected_feed,
            },
            LifecycleEvent::Wipe {
                motion: actual_motion,
                extrusion: actual_extrusion,
                feed: actual_feed,
            },
        ) => {
            expected_motion.command == actual_motion.command
                && position_matches(&expected_motion.start, &actual_motion.start)
                && position_matches(&expected_motion.end, &actual_motion.end)
                && expected_motion.arc_center == actual_motion.arc_center
                && expected_motion.turns == actual_motion.turns
                && numeric_matches(expected_extrusion, actual_extrusion, 0.000011)
                && expected_feed == actual_feed
        }
        _ => false,
    }
}

fn position_matches(expected: &super::model::Position, actual: &super::model::Position) -> bool {
    numeric_matches(&expected.x, &actual.x, 0.001)
        && numeric_matches(&expected.y, &actual.y, 0.001)
        && expected.z == actual.z
}

fn numeric_matches(expected: &str, actual: &str, tolerance: f64) -> bool {
    let Ok(expected) = expected.parse::<f64>() else {
        return expected == actual;
    };
    let Ok(actual) = actual.parse::<f64>() else {
        return false;
    };
    (expected - actual).abs() <= tolerance
}
