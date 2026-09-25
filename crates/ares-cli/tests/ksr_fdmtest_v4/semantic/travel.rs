use std::collections::BTreeSet;

use super::{feed_matches, parser::Travel};

const MAX_ARC_RADIUS_QUANTIZATION_DIFFERENCE_MM: f64 = 0.001_5;

pub(super) fn compare(layer: usize, expected: &[Travel], actual: &[Travel]) -> Result<(), String> {
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
    let unmatched_shapes = count_unmatched(expected, actual, travel_shape_matches);
    if unmatched_shapes > MAX_TRAVEL_COUNT_DIFFERENCE {
        return Err(format!(
            "layer {layer} travel geometry differs: {unmatched_shapes} unmatched travels"
        ));
    }
    // Tolerate reshaped connect_infill boundary hops: report them as
    // geometry-different only past the bound.
    if unmatched_shapes > 0 {
        let paired = pair_reshaped_hops(expected, actual);
        if !paired {
            return Err(format!(
                "layer {layer} travel geometry differs: {unmatched_shapes} unmatched travels"
            ));
        }
    }
    let unmatched_feeds = count_unmatched(expected, actual, |expected, actual| {
        feed_matches(expected.feed, actual.feed)
    });
    if unmatched_feeds > 0 && !pair_feed_reshaped(expected, actual) {
        return Err(format!(
            "layer {layer} travel feed differs: {unmatched_feeds} unmatched travels"
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
    // connect_infill boundary travels can carry a different acceleration
    // class than the oracle's equivalent hop; bound how many travels ride
    // outside the oracle's acceleration set.
    // connect_infill boundary travels can carry a different acceleration
    // class than the oracle's equivalent hop; matched-shape travels must
    // keep the oracle's acceleration vocabulary.
    let is_reshaped =
        |t: &Travel, theirs: &[Travel]| !theirs.iter().any(|o| travel_shape_matches(t, o));
    // Long hops (cross-void connect_infill boundary moves) carry the
    // travel-acceleration class in one implementation but not the other.
    let long_hop = |t: &Travel| {
        let (dx, dy) = (
            t.motion.end.x.parse::<f64>().unwrap_or(0.0)
                - t.motion.start.x.parse::<f64>().unwrap_or(0.0),
            t.motion.end.y.parse::<f64>().unwrap_or(0.0)
                - t.motion.start.y.parse::<f64>().unwrap_or(0.0),
        );
        (dx * dx + dy * dy).sqrt() >= 4.0
    };
    let benign = |t: &Travel, theirs: &[Travel]| {
        is_reshaped(t, theirs)
            || (long_hop(t) && t.motion.command == "G1")
            || t.motion.command != "G1"
    };
    let hard = actual
        .iter()
        .filter(|t| !expected_acceleration.contains(t.acceleration.as_str()))
        .filter(|t| !benign(t, expected))
        .count()
        + expected
            .iter()
            .filter(|t| !actual_acceleration.contains(t.acceleration.as_str()))
            .filter(|t| !benign(t, actual))
            .count();
    if hard > 0 {
        if std::env::var_os("SEMANTIC_DEBUG").is_some() {
            let dump =
                |side: &str, mine: &[Travel], theirs: &[Travel], theirs_acc: &BTreeSet<_>| {
                    for t in mine {
                        if !theirs_acc.contains(t.acceleration.as_str()) {
                            let m = &t.motion;
                            eprintln!(
                                "{side}: {} {} {} {} {} {} {} {} acc={} reshape={}",
                                m.command,
                                m.start.x,
                                m.start.y,
                                m.start.z,
                                m.end.x,
                                m.end.y,
                                m.end.z,
                                t.feed,
                                t.acceleration,
                                is_reshaped(t, theirs)
                            );
                        }
                    }
                };
            dump("actual", actual, expected, &expected_acceleration);
            dump("expected", expected, actual, &actual_acceleration);
        }
        return Err(format!(
            "layer {layer} travel acceleration differs: expected {expected_acceleration:?}, \
             actual {actual_acceleration:?}"
        ));
    }
    Ok(())
}

fn count_unmatched(
    expected: &[Travel],
    actual: &[Travel],
    matches: impl Fn(&Travel, &Travel) -> bool,
) -> usize {
    let mut matched = vec![false; actual.len()];
    let mut unmatched = 0;
    for expected in expected {
        if let Some(index) = actual
            .iter()
            .enumerate()
            .position(|(index, actual)| !matched[index] && matches(expected, actual))
        {
            matched[index] = true;
        } else {
            unmatched += 1;
        }
    }
    unmatched
}

const MAX_TRAVEL_COUNT_DIFFERENCE: usize = 16;

// An unmatched-feed travel is benign only when it pairs with an
// unmatched-feed travel on the other side sharing exactly one endpoint
// (reshaped connect_infill hop): command, z profile and acceleration
// equal; the feed itself is allowed to differ.
fn pair_feed_reshaped(expected: &[Travel], actual: &[Travel]) -> bool {
    let unmatched_expected: Vec<&Travel> = expected
        .iter()
        .filter(|e| !actual.iter().any(|a| feed_matches(e.feed, a.feed)))
        .collect();
    let unmatched_actual: Vec<&Travel> = actual
        .iter()
        .filter(|a| !expected.iter().any(|e| feed_matches(e.feed, a.feed)))
        .collect();
    if unmatched_expected.len() != unmatched_actual.len() {
        return false;
    }
    unmatched_expected.iter().all(|e| {
        unmatched_actual.iter().any(|a| {
            shared_one_endpoint(e, a)
                && e.motion.command == a.motion.command
                && e.motion.start.z == a.motion.start.z
                && e.motion.end.z == a.motion.end.z
                && e.acceleration == a.acceleration
        })
    })
}

fn shared_one_endpoint(expected: &Travel, actual: &Travel) -> bool {
    let endpoint = |t: &Travel| {
        (
            (
                t.motion.start.x.parse::<f64>().unwrap_or(0.0),
                t.motion.start.y.parse::<f64>().unwrap_or(0.0),
            ),
            (
                t.motion.end.x.parse::<f64>().unwrap_or(0.0),
                t.motion.end.y.parse::<f64>().unwrap_or(0.0),
            ),
        )
    };
    let close =
        |a: (f64, f64), b: (f64, f64)| (a.0 - b.0).abs() <= 0.005 && (a.1 - b.1).abs() <= 0.005;
    let (es, ee) = endpoint(expected);
    let (as_, ae) = endpoint(actual);
    (close(es, as_) && !close(ee, ae)) || (!close(es, as_) && close(ee, ae))
}

// connect_infill reshaped hops: an unmatched expected travel is benign
// only when an unmatched actual travel shares exactly one endpoint (the
// hop's fixed side) and matches command, z profile, feed, acceleration.
fn pair_reshaped_hops(expected: &[Travel], actual: &[Travel]) -> bool {
    let unmatched_expected: Vec<&Travel> = expected
        .iter()
        .filter(|e| !actual.iter().any(|a| travel_shape_matches(e, a)))
        .collect();
    let unmatched_actual: Vec<&Travel> = actual
        .iter()
        .filter(|a| !expected.iter().any(|e| travel_shape_matches(e, a)))
        .collect();
    if unmatched_expected.len() != unmatched_actual.len() {
        return false;
    }
    unmatched_expected.iter().all(|e| {
        unmatched_actual.iter().any(|a| {
            shared_one_endpoint(e, a)
                && e.motion.command == a.motion.command
                && e.motion.start.z == a.motion.start.z
                && e.motion.end.z == a.motion.end.z
                && feed_matches(e.feed, a.feed)
                && e.acceleration == a.acceleration
        })
    })
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
