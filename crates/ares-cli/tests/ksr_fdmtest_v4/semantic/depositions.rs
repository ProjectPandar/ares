use super::{feed_matches, motion_matches, numeric_matches, parser::Deposition};

pub(super) fn compare_deposition(
    layer: usize,
    expected: &[Deposition],
    actual: &[Deposition],
) -> Result<(), String> {
    let difference = expected.len().abs_diff(actual.len());
    if difference > MAX_SAME_TARGET_DEPOSITION_COUNT_DIFFERENCE {
        return Err(format!(
            "layer {layer} deposition count differs: expected {}, actual {}",
            expected.len(),
            actual.len()
        ));
    }
    // Greedy multiset matching: quantization boundaries can split or merge
    // a nearly-degenerate deposition (oracle emits X=a twice where ares
    // emits a then a+1µm), shifting every later index. Match each expected
    // deposition to any unmatched actual within tolerance.
    let mut used = vec![false; actual.len()];
    let mut unmatched_expected = Vec::new();
    'outer: for deposition in expected {
        for (index, candidate) in actual.iter().enumerate() {
            if !used[index] && deposition_matches(deposition, candidate) {
                used[index] = true;
                continue 'outer;
            }
        }
        unmatched_expected.push(deposition);
    }
    let leftovers: Vec<&Deposition> = actual
        .iter()
        .zip(&used)
        .filter_map(|(d, &u)| (!u).then_some(d))
        .collect();
    let unmatched = !unmatched_expected.is_empty() || !leftovers.is_empty();
    let degenerate = |d: &&Deposition| {
        d.extrusion.parse::<f64>().unwrap_or(f64::MAX).abs() <= MAX_LEFTOVER_DEPOSITION_EXTRUSION_MM
    };
    if !unmatched {
        return Ok(());
    }
    if unmatched_expected.iter().all(degenerate) && leftovers.iter().all(degenerate) {
        return Ok(());
    }
    // `connect_infill` chains sparse-infill segments with upstream
    // `std::sort`, whose output is unstable across implementations: the
    // same segment set produces different merge boundaries, so per-
    // deposition identity is not meaningful there. The sound invariant is
    // the deposited filament total per feature per layer.
    let unchained = |f: &str| {
        matches!(
            f,
            "Sparse infill"
                | "Internal infill"
                | "Internal solid infill"
                | "Infill"
                | "Gap infill"
                | "Gap fill"
        )
    };
    let sum = |list: &[&Deposition]| {
        list.iter()
            .filter(|d| unchained(&d.feature))
            .map(|d| d.extrusion.parse::<f64>().unwrap_or(0.0))
            .sum::<f64>()
    };
    let unchained_delta = (sum(&unmatched_expected) - sum(&leftovers)).abs();
    let chained_unmatched = unmatched_expected
        .iter()
        .chain(&leftovers)
        .any(|d| !unchained(&d.feature));
    // Bound relative to the feature's deposited total in this layer so
    // large sparse-infill layers keep the same effective precision.
    let scale = (sum(&unmatched_expected) + sum(&leftovers)).max(1.0);
    let allowed =
        MAX_UNCHAINED_EXTRUSION_MISMATCH_MM.max(scale * MAX_UNCHAINED_EXTRUSION_MISMATCH_RATIO);
    if chained_unmatched || unchained_delta > allowed {
        return Err(format!(
            "layer {layer} deposition 1 differs: {} unmatched expected, \
             {} unmatched actual (extrusion delta {unchained_delta:.4}mm, \
             first {:?} vs {:?})",
            unmatched_expected.len(),
            leftovers.len(),
            unmatched_expected.first(),
            leftovers.first()
        ));
    }
    Ok(())
}

// Same-target tolerances: sub-µm Coord rounding drifts the last emitted
// decimal; arc-fit centers on near-straight arcs (radius ~287mm) can
// differ by up to ~0.6mm without changing the deposited shape.
const MAX_SAME_TARGET_COORDINATE_DIFFERENCE_MM: f64 = 0.005;
const MAX_SAME_TARGET_EXTRUSION_DIFFERENCE_MM: f64 = 0.001;
const MAX_SAME_TARGET_ARC_CENTER_DIFFERENCE_MM: f64 = 0.6;
const MAX_SAME_TARGET_DEPOSITION_COUNT_DIFFERENCE: usize = 64;
// Depositions that quantize to a merge/split boundary only shift a tiny
// extrusion amount, so unmatched leftovers must stay near-degenerate.
const MAX_LEFTOVER_DEPOSITION_EXTRUSION_MM: f64 = 0.02;
// connect_infill std::sort instability redistributes sparse-infill
// extrusion across merge boundaries; merged moves also drop the tiny
// overlap between joined segments, so bound the net deposited total
// loosely (measured ~0.06mm/layer on ksr_fdmtest_v4).
const MAX_UNCHAINED_EXTRUSION_MISMATCH_MM: f64 = 0.4;
const MAX_UNCHAINED_EXTRUSION_MISMATCH_RATIO: f64 = 0.05;

const MAX_CROSS_TARGET_DEPOSITION_COORDINATE_DIFFERENCE_MM: f64 = 0.15;
const MAX_CROSS_TARGET_DEPOSITION_EXTRUSION_DIFFERENCE_MM: f64 = 0.005;
const MAX_CROSS_TARGET_SPLIT_EXTRUSION_MM: f64 = 0.0015;
const MAX_CROSS_TARGET_DEPOSITION_FEED_DIFFERENCE: f64 = 20.0;

pub(super) fn compare_deposition_cross_target(
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
        if !deposition_matches_cross_target(expected, actual) {
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
        && feed_matches(expected.feed, actual.feed)
        && numeric_matches(
            expected.extrusion.parse().unwrap(),
            actual.extrusion.parse().unwrap(),
            MAX_SAME_TARGET_EXTRUSION_DIFFERENCE_MM,
        )
        && motion_matches(
            &expected.motion,
            &actual.motion,
            MAX_SAME_TARGET_COORDINATE_DIFFERENCE_MM,
            MAX_SAME_TARGET_ARC_CENTER_DIFFERENCE_MM,
        )
}

fn deposition_matches_cross_target(expected: &Deposition, actual: &Deposition) -> bool {
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
        && motion_matches(
            &expected.motion,
            &actual.motion,
            MAX_CROSS_TARGET_DEPOSITION_COORDINATE_DIFFERENCE_MM,
            MAX_CROSS_TARGET_DEPOSITION_COORDINATE_DIFFERENCE_MM,
        )
}

fn cross_target_feed_matches(expected: f64, actual: f64) -> bool {
    let absolute = (expected - actual).abs();
    let relative = absolute / expected.abs().max(f64::EPSILON);
    absolute <= MAX_CROSS_TARGET_DEPOSITION_FEED_DIFFERENCE && relative <= 0.01
}
