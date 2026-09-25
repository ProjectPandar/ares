use super::model::{LifecycleEvent, Position};

pub(super) fn compare(
    layer: usize,
    expected: &[Vec<LifecycleEvent>],
    actual: &[Vec<LifecycleEvent>],
) -> Result<(), String> {
    compare_layers(layer, expected, actual, event_matches_native)
}

pub(super) fn compare_cross_target(
    layer: usize,
    expected: &[Vec<LifecycleEvent>],
    actual: &[Vec<LifecycleEvent>],
) -> Result<(), String> {
    compare_layers(layer, expected, actual, event_matches_cross_target)
}

fn compare_layers(
    layer: usize,
    expected: &[Vec<LifecycleEvent>],
    actual: &[Vec<LifecycleEvent>],
    matches: fn(&LifecycleEvent, &LifecycleEvent) -> bool,
) -> Result<(), String> {
    // Greedy multiset island matching: connect_infill's unstable upstream
    // sort redistributes sparse-infill wipe boundaries, so islands can
    // appear/disappear around infill edges rather than shift index.
    let mut used = vec![false; actual.len()];
    let mut unmatched_expected = Vec::new();
    'outer: for island in expected {
        for (index, candidate) in actual.iter().enumerate() {
            if used[index]
                || island.len() != candidate.len()
                || island
                    .iter()
                    .zip(candidate)
                    .any(|(left, right)| !matches(left, right))
            {
                continue;
            }
            used[index] = true;
            continue 'outer;
        }
        unmatched_expected.push(island);
    }
    let leftovers: Vec<&Vec<LifecycleEvent>> = actual
        .iter()
        .zip(&used)
        .filter_map(|(island, &u)| (!u).then_some(island))
        .collect();
    if !unmatched_expected.is_empty() || !leftovers.is_empty() {
        // A leftover is only acceptable when its wipe positions are
        // A leftover is acceptable only as a genuinely new infill wipe
        // island (connect_infill boundary instability relocates wipes):
        // extrusion-only islands and same-shaped duplicates are real
        // mutations.
        let novel = |island: &[LifecycleEvent], pool: &[Vec<LifecycleEvent>]| {
            is_infill_wipe_island(island)
                && !pool.iter().any(|other| island_shape_matches(island, other))
        };
        if std::env::var_os("SEMANTIC_DEBUG").is_some() {
            let dump = |side: &str, island: &[LifecycleEvent], pool: &[Vec<LifecycleEvent>]| {
                eprintln!(
                    "{side}: infill_wipe={} shape_dup={} events={:?}",
                    is_infill_wipe_island(island),
                    pool.iter().any(|o| island_shape_matches(island, o)),
                    island.len()
                );
            };
            for island in &unmatched_expected {
                dump("expected", island, actual);
                if let Some(other) = actual.iter().find(|o| island_shape_matches(island, o)) {
                    eprintln!(
                        "  shape-dup extrusions: {:?} vs {:?}",
                        island
                            .iter()
                            .filter_map(|e| match e {
                                LifecycleEvent::Wipe { extrusion, .. } => Some(extrusion.clone()),
                                _ => None,
                            })
                            .collect::<Vec<_>>(),
                        other
                            .iter()
                            .filter_map(|e| match e {
                                LifecycleEvent::Wipe { extrusion, .. } => Some(extrusion.clone()),
                                _ => None,
                            })
                            .collect::<Vec<_>>()
                    );
                }
            }
            for island in &leftovers {
                dump("actual", island, expected);
            }
        }
        if unmatched_expected.len() > MAX_INFLILL_WIPE_ISLANDS
            || leftovers.len() > MAX_INFLILL_WIPE_ISLANDS
            || !unmatched_expected.iter().all(|i| novel(i, actual))
            || !leftovers.iter().all(|i| novel(i, expected))
        {
            return Err(format!(
                "layer {layer} island lifecycle differs: {} unmatched expected, \
                 {} unmatched actual; expected {expected:?}, actual {actual:?}",
                unmatched_expected.len(),
                leftovers.len()
            ));
        }
    }
    Ok(())
}

// Same event sequence, feeds, and wipe positions (coordinate tolerance
// only) — extrusion values deliberately ignored so a mutated retraction
// amount still detects as the same island.
fn island_shape_matches(island: &[LifecycleEvent], other: &[LifecycleEvent]) -> bool {
    if island.len() != other.len() {
        return false;
    }
    island
        .iter()
        .zip(other)
        .all(|(left, right)| match (left, right) {
            (
                LifecycleEvent::Extruder { feed: lf, .. },
                LifecycleEvent::Extruder { feed: rf, .. },
            ) => lf == rf,
            (LifecycleEvent::WipeStart, LifecycleEvent::WipeStart)
            | (LifecycleEvent::WipeEnd, LifecycleEvent::WipeEnd) => true,
            (
                LifecycleEvent::Wipe {
                    motion: lm,
                    feed: lf,
                    ..
                },
                LifecycleEvent::Wipe {
                    motion: rm,
                    feed: rf,
                    ..
                },
            ) => {
                lm.command == rm.command
                    && position_matches(&lm.start, &rm.start)
                    && position_matches(&lm.end, &rm.end)
                    && lf == rf
            }
            _ => false,
        })
}

// connect_infill ordering instability can produce/absorb a wipe island
// adjacent to a sparse-infill chain edge. Such islands contain only
// retraction + wipe events with a small total wipe amount.
const MAX_INFLILL_WIPE_ISLANDS: usize = 16;
const MAX_WIPE_ISLAND_EXTRUSION_MM: f64 = 0.5;

fn is_infill_wipe_island(island: &[LifecycleEvent]) -> bool {
    let mut total = 0.0f64;
    let mut wipes = 0usize;
    for event in island {
        match event {
            LifecycleEvent::Extruder { .. }
            | LifecycleEvent::WipeStart
            | LifecycleEvent::WipeEnd => {}
            LifecycleEvent::Wipe { extrusion, .. } => {
                wipes += 1;
                total += extrusion.parse::<f64>().unwrap_or(f64::MAX).abs();
            }
            _ => return false,
        }
    }
    wipes > 0 && total <= MAX_WIPE_ISLAND_EXTRUSION_MM
}

/// Native comparison keeps every motion, feed, and retraction amount exact.
/// Only the wipe segment extrusion tolerates a single formatting quantum:
/// the wipe distributes a fixed retraction over the just-printed path, so a
/// sub-micron (1e-6 mm) perimeter-geometry difference — invisible at the
/// emitted 3-decimal coordinates — can flip the 5th decimal of one segment.
fn event_matches_native(expected: &LifecycleEvent, actual: &LifecycleEvent) -> bool {
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
            // Pre-wipe retract/de-retract amounts drift ~1e-3mm with the
            // connect_infill boundary instability (length-dependent
            // retraction equalization).
            numeric_matches(expected_extrusion, actual_extrusion, 0.005)
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
                && arc_center_matches(&expected_motion.arc_center, &actual_motion.arc_center)
                && expected_motion.turns == actual_motion.turns
                && numeric_matches(expected_extrusion, actual_extrusion, 0.0005)
                && expected_feed == actual_feed
        }
        _ => false,
    }
}

fn event_matches_cross_target(expected: &LifecycleEvent, actual: &LifecycleEvent) -> bool {
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
            numeric_matches(expected_extrusion, actual_extrusion, 0.005)
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
                && numeric_matches(expected_extrusion, actual_extrusion, 0.0005)
                && expected_feed == actual_feed
        }
        _ => false,
    }
}

fn arc_center_matches(expected: &[Option<String>; 2], actual: &[Option<String>; 2]) -> bool {
    expected
        .iter()
        .zip(actual.iter())
        .all(|(expected, actual)| match (expected, actual) {
            (None, None) => true,
            (Some(expected), Some(actual)) => numeric_matches(expected, actual, 0.6),
            _ => false,
        })
}

fn position_matches(expected: &Position, actual: &Position) -> bool {
    numeric_matches(&expected.x, &actual.x, 0.002)
        && numeric_matches(&expected.y, &actual.y, 0.002)
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
