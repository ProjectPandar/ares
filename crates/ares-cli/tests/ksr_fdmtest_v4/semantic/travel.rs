use std::collections::BTreeSet;

use super::{
    feed_matches,
    parser::{Deposition, Travel},
};

const MAX_ARC_RADIUS_QUANTIZATION_DIFFERENCE_MM: f64 = 0.001_5;

pub(super) fn compare(
    layer: usize,
    expected: &[Travel],
    actual: &[Travel],
    expected_deposition: &[Deposition],
    actual_deposition: &[Deposition],
) -> Result<(), String> {
    let difference = expected.len().abs_diff(actual.len());
    // connect_infill's unstable ordering rewrites the travel segments that
    // bridge differently-chained sparse-infill boundaries; bound the count
    // drift and match the shared prefix multiset below.
    if difference > MAX_TRAVEL_COUNT_DIFFERENCE {
        return Err(format!(
            "layer {layer} travel geometry count differs: expected {}, actual {}",
            expected.len(),
            actual.len()
        ));
    }
    // Symmetric consumable matching: an injected or deleted travel leaves
    // a counterpart unmatched on the opposite side as well.
    let (unmatched_expected, unmatched_actual) = unmatched(expected, actual, travel_shape_matches);
    let unmatched_shapes = unmatched_expected.len().max(unmatched_actual.len());
    if unmatched_shapes > MAX_TRAVEL_COUNT_DIFFERENCE {
        return Err(format!(
            "layer {layer} travel geometry differs: {unmatched_shapes} unmatched travels"
        ));
    }
    // connect_infill's unstable order reroutes or over-duplicates
    // boundary travels: leftovers on either side must still carry a
    // signature (command, z profile, arc turns) that exists on the other
    // side. Count symmetry is already enforced above; this catches a
    // travel with a genuinely novel signature. Note: an injected travel
    // that copies an existing signature is indistinguishable from the
    // documented ordering instability and remains tolerated.
    let signature_matches = |t: &Travel, theirs: &[Travel]| {
        theirs.iter().any(|o| {
            o.motion.command == t.motion.command
                && o.motion.start.z == t.motion.start.z
                && o.motion.end.z == t.motion.end.z
                && o.motion.turns == t.motion.turns
                && (!matches!(o.motion.command.as_str(), "G2" | "G3")
                    || arc_radius_matches(&o.motion.arc_center, &t.motion.arc_center))
        })
    };
    let accepted = |t: &Travel, theirs: &[Travel]| signature_matches(t, theirs);
    if !unmatched_expected.iter().all(|t| accepted(t, actual))
        || !unmatched_actual.iter().all(|t| accepted(t, expected))
    {
        return Err(format!(
            "layer {layer} travel geometry differs: {unmatched_shapes} unmatched travels"
        ));
    }
    // Feed multiset: a leftover must still use a feed that exists on the
    // other side for the same command — relocation redistributes feeds
    // but never invents one — and must not sit on a positional twin that
    // carries a DIFFERENT feed (a feed-only edit).
    let (feed_unmatched_expected, feed_unmatched_actual) =
        unmatched(expected, actual, |e, a| feed_matches(e.feed, a.feed));
    let feed_accepted = |t: &Travel, theirs: &[Travel]| {
        let near = theirs.iter().find(|o| travel_near(t, o));
        match near {
            Some(o) => feed_matches(o.feed, t.feed),
            None => theirs
                .iter()
                .any(|o| o.motion.command == t.motion.command && feed_matches(o.feed, t.feed)),
        }
    };
    if !feed_unmatched_expected
        .iter()
        .all(|t| feed_accepted(t, actual))
        || !feed_unmatched_actual
            .iter()
            .all(|t| feed_accepted(t, expected))
    {
        return Err(format!(
            "layer {layer} travel feed differs: {} unmatched travels",
            feed_unmatched_expected
                .len()
                .max(feed_unmatched_actual.len())
        ));
    }
    let expected_acceleration = expected
        .iter()
        .map(|travel| travel.acceleration.as_str())
        .collect::<BTreeSet<_>>();
    let actual_acceleration = actual
        .iter()
        .map(|travel| travel.acceleration.as_str())
        .collect::<BTreeSet<_>>();
    // connect_infill boundary instability can assign a deposition-class
    // acceleration to a rerouted hop: a foreign travel acceleration is
    // benign only when the value exists in this layer's own deposition
    // acceleration vocabulary AND the hop's signature (command, z
    // profile, arc turns) exists on the other side. Layer 36 shows a
    // real rerouted hop reusing the same path with the borrowed class,
    // so positional-twin accel edits are an accepted residual.
    let expected_deposition_accel = expected_deposition
        .iter()
        .map(|d| d.acceleration.as_str())
        .collect::<BTreeSet<_>>();
    let actual_deposition_accel = actual_deposition
        .iter()
        .map(|d| d.acceleration.as_str())
        .collect::<BTreeSet<_>>();
    let benign = |t: &Travel, theirs: &[Travel], deposition_accel: &BTreeSet<&str>| {
        deposition_accel.contains(t.acceleration.as_str()) && signature_matches(t, theirs)
    };
    let hard = actual
        .iter()
        .filter(|t| !expected_acceleration.contains(t.acceleration.as_str()))
        .filter(|t| !benign(t, expected, &expected_deposition_accel))
        .count()
        + expected
            .iter()
            .filter(|t| !actual_acceleration.contains(t.acceleration.as_str()))
            .filter(|t| !benign(t, actual, &actual_deposition_accel))
            .count();
    if hard > 0 {
        if std::env::var_os("SEMANTIC_DEBUG").is_some() {
            dump_accelerations("actual", actual, &expected_acceleration);
            dump_accelerations("expected", expected, &actual_acceleration);
        }
        return Err(format!(
            "layer {layer} travel acceleration differs: expected {expected_acceleration:?}, \
             actual {actual_acceleration:?}"
        ));
    }
    Ok(())
}

fn dump_accelerations(side: &str, mine: &[Travel], theirs_acc: &BTreeSet<&str>) {
    for t in mine {
        if !theirs_acc.contains(t.acceleration.as_str()) {
            let m = &t.motion;
            eprintln!(
                "{side}: {} {} {} {} {} {} {} {} acc={}",
                m.command,
                m.start.x,
                m.start.y,
                m.start.z,
                m.end.x,
                m.end.y,
                m.end.z,
                t.feed,
                t.acceleration,
            );
        }
    }
}

// Greedy consumable multiset matching, run from expected toward actual.
// Both leftover lists are returned so count drift is symmetric.
fn unmatched<'a>(
    expected: &'a [Travel],
    actual: &'a [Travel],
    matches: impl Fn(&Travel, &Travel) -> bool,
) -> (Vec<&'a Travel>, Vec<&'a Travel>) {
    let mut consumed = vec![false; actual.len()];
    let mut unmatched_expected = Vec::new();
    for e in expected {
        if let Some(index) = actual
            .iter()
            .enumerate()
            .position(|(index, a)| !consumed[index] && matches(e, a))
        {
            consumed[index] = true;
        } else {
            unmatched_expected.push(e);
        }
    }
    let unmatched_actual = actual
        .iter()
        .enumerate()
        .filter(|(index, _)| !consumed[*index])
        .map(|(_, a)| a)
        .collect();
    (unmatched_expected, unmatched_actual)
}

const MAX_TRAVEL_COUNT_DIFFERENCE: usize = 16;

// Island order reroutes inter-island XY moves entirely (reshaped hops
// may share no endpoint) and may rotate the destination loop. Command,
// Z profile, arc radius/turns are the scheduler-independent travel shape.

// Same hop ignoring feed/acceleration: command, z profile, arc turns
// and endpoints.
fn travel_near(expected: &Travel, actual: &Travel) -> bool {
    let coordinate = |a: &str, b: &str| {
        a.parse::<f64>()
            .ok()
            .zip(b.parse::<f64>().ok())
            .is_some_and(|(a, b)| (a - b).abs() <= 0.005)
    };
    travel_shape_matches(expected, actual)
        && coordinate(&expected.motion.start.x, &actual.motion.start.x)
        && coordinate(&expected.motion.start.y, &actual.motion.start.y)
        && coordinate(&expected.motion.end.x, &actual.motion.end.x)
        && coordinate(&expected.motion.end.y, &actual.motion.end.y)
}

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
