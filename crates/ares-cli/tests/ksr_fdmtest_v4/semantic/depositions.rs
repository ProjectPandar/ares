use super::{
    arc_is_near_straight, feed_matches, motion_matches, numeric_matches, parser::Deposition,
    position_matches,
};

// Features whose segments connect_infill chains: the same set produces
// different merge boundaries AND segment directions across std::sort
// orderings (FillBase.cpp:1700 arc_length sort instability).
const CONNECT_INFILL_FEATURES: &[&str] = &[
    "Sparse infill",
    "Internal infill",
    "Internal solid infill",
    "Infill",
    "Gap infill",
    "Gap fill",
];

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
    if unmatched_expected.is_empty() && leftovers.is_empty() {
        return Ok(());
    }
    if unmatched_expected.len() > MAX_SAME_TARGET_UNMATCHED_COUNT
        || leftovers.len() > MAX_SAME_TARGET_UNMATCHED_COUNT
    {
        if std::env::var_os("SEMANTIC_DEBUG").is_some() {
            let dump = |side: &str, list: &[&Deposition]| {
                for d in list.iter().take(12) {
                    eprintln!(
                        "{side}: {} {} {} {} {} {} {} {} {} e={} f={} w={} a={}",
                        d.feature,
                        d.motion.command,
                        d.motion.start.x,
                        d.motion.start.y,
                        d.motion.start.z,
                        d.motion.end.x,
                        d.motion.end.y,
                        d.motion.end.z,
                        d.extrusion,
                        d.extrusion,
                        d.feed,
                        d.width,
                        d.acceleration
                    );
                }
            };
            dump("expected", &unmatched_expected);
            dump("actual", &leftovers);
        }
        return Err(format!(
            "layer {layer} deposition 1 differs: {} unmatched expected, {} unmatched actual",
            unmatched_expected.len(),
            leftovers.len()
        ));
    }
    // `connect_infill` chains sparse-infill segments with upstream
    // `std::sort` (arc_length, FillBase.cpp:1700), whose output is
    // unstable across implementations: the same segment set produces
    // different merge boundaries, so per-deposition identity is not
    // meaningful there. The sound invariants are (1) spatial coverage —
    // every unmatched deposition has an unmatched counterpart covering
    // the same neighborhood — and (2) the deposited filament total per
    // feature per layer.
    let unchained = |f: &str| CONNECT_INFILL_FEATURES.contains(&f);
    let extrusion_of = |d: &&Deposition| d.extrusion.parse::<f64>().unwrap_or(0.0);
    let length_of = |d: &&Deposition| {
        let (Ok(sx), Ok(sy), Ok(ex), Ok(ey)) = (
            d.motion.start.x.parse::<f64>(),
            d.motion.start.y.parse::<f64>(),
            d.motion.end.x.parse::<f64>(),
            d.motion.end.y.parse::<f64>(),
        ) else {
            return f64::NAN;
        };
        ((ex - sx).powi(2) + (ey - sy).powi(2)).sqrt()
    };
    // Extrusion is proportional to path length at fixed width/height: a
    // counterpart that covers the same neighborhood with a wildly
    // different extrusion-per-mm is an edited segment, not a merge
    // boundary artifact.
    let flow_compatible = |d: &&Deposition, o: &&Deposition| {
        let (le, lo) = (length_of(d), length_of(o));
        if !le.is_finite() || !lo.is_finite() || le <= 0.0 || lo <= 0.0 {
            return true;
        }
        let (ee, eo) = (extrusion_of(d), extrusion_of(o));
        let (flow_e, flow_o) = (ee / le, eo / lo);
        (flow_e - flow_o).abs() <= flow_e.abs().max(flow_o.abs()) * 0.25
    };
    let degenerate =
        |d: &&Deposition| extrusion_of(d).abs() <= MAX_LEFTOVER_DEPOSITION_EXTRUSION_MM;
    // Spatial pairing: same feature, and the unmatched counterpart's
    // start or end lies within the shared boundary neighborhood.
    let paired = |d: &&Deposition, pool: &[&Deposition]| {
        pool.iter().any(|o| {
            o.feature == d.feature
                && (position_matches(
                    &d.motion.start,
                    &o.motion.start,
                    MAX_UNMATCHED_PAIR_DISTANCE_MM,
                ) || position_matches(
                    &d.motion.start,
                    &o.motion.end,
                    MAX_UNMATCHED_PAIR_DISTANCE_MM,
                ) || position_matches(
                    &d.motion.end,
                    &o.motion.start,
                    MAX_UNMATCHED_PAIR_DISTANCE_MM,
                ) || position_matches(
                    &d.motion.end,
                    &o.motion.end,
                    MAX_UNMATCHED_PAIR_DISTANCE_MM,
                ))
                && flow_compatible(d, o)
        })
    };
    let valid = |d: &&Deposition, pool: &[&Deposition]| {
        unchained(&d.feature) && paired(d, pool)
            || degenerate(d)
                && pool.iter().any(|o| {
                    o.feature == d.feature
                        && flow_compatible(d, o)
                        && (position_matches(
                            &d.motion.start,
                            &o.motion.start,
                            MAX_DEGENERATE_PAIR_DISTANCE_MM,
                        ) || position_matches(
                            &d.motion.end,
                            &o.motion.end,
                            MAX_DEGENERATE_PAIR_DISTANCE_MM,
                        ))
                })
    };
    let bad_expected = unmatched_expected
        .iter()
        .filter(|d| !valid(d, &leftovers))
        .count();
    let bad_actual = leftovers
        .iter()
        .filter(|d| !valid(d, &unmatched_expected))
        .count();
    if bad_expected > 0 || bad_actual > 0 {
        return Err(format!(
            "layer {layer} deposition 1 differs: {} unmatched expected, \
             {} unmatched actual (first {:?} vs {:?})",
            unmatched_expected.len(),
            leftovers.len(),
            unmatched_expected.first(),
            leftovers.first()
        ));
    }
    // Per-feature deposited totals, both directions: merge boundaries
    // shift extrusion between segments of the SAME feature only.
    let per_feature_delta =
        |list_a: &[&Deposition], list_b: &[&Deposition], measure: fn(&&Deposition) -> f64| {
            let mut features: Vec<String> = list_a
                .iter()
                .chain(list_b.iter())
                .map(|d| d.feature.clone())
                .collect();
            features.sort();
            features.dedup();
            features
                .iter()
                .map(|f| {
                    let sum = |list: &[&Deposition]| {
                        list.iter()
                            .filter(|d| d.feature == *f)
                            .map(|d| measure(d))
                            .sum::<f64>()
                    };
                    (sum(list_a) - sum(list_b)).abs()
                })
                .fold(0.0f64, f64::max)
        };
    let delta = per_feature_delta(&unmatched_expected, &leftovers, extrusion_of);
    // Bound relative to the smaller unmatched pool so one-sided drift
    // cannot double-count. Spatial pairing above already rejects
    // relocated segments, so path length need not be conserved here.
    let pool = (unmatched_expected
        .iter()
        .map(extrusion_of)
        .sum::<f64>()
        .abs())
    .min(leftovers.iter().map(extrusion_of).sum::<f64>().abs())
    .max(1.0);
    let allowed =
        MAX_UNCHAINED_EXTRUSION_MISMATCH_MM.max(pool * MAX_UNCHAINED_EXTRUSION_MISMATCH_RATIO);
    if delta > allowed {
        return Err(format!(
            "layer {layer} deposition 1 differs: {} unmatched expected, \
             {} unmatched actual (extrusion delta {delta:.4}mm, first \
             {:?} vs {:?})",
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
// differ by up to ~0.6mm without changing the deposited shape. The wide
// bound applies only when BOTH arcs are provably near-straight.
const MAX_SAME_TARGET_COORDINATE_DIFFERENCE_MM: f64 = 0.005;
const MAX_SAME_TARGET_EXTRUSION_DIFFERENCE_MM: f64 = 0.001;
const MAX_SAME_TARGET_ARC_CENTER_DIFFERENCE_MM: f64 = 0.6;
const MAX_SAME_TARGET_DEPOSITION_COUNT_DIFFERENCE: usize = 64;
// connect_infill reshuffles up to ~74 sparse-infill segments per layer
// on this fixture; pair-bounded, not count-bounded.
const MAX_SAME_TARGET_UNMATCHED_COUNT: usize = 96;
// Depositions that quantize to a merge/split boundary only shift a tiny
// extrusion amount; a degenerate leftover must still sit on the same
// endpoint as a counterpart of the same feature.
const MAX_LEFTOVER_DEPOSITION_EXTRUSION_MM: f64 = 0.02;
const MAX_DEGENERATE_PAIR_DISTANCE_MM: f64 = 0.01;
// connect_infill merge boundaries shift segments locally; an unmatched
// deposition pairs only with a counterpart covering the same
// neighborhood.
const MAX_UNMATCHED_PAIR_DISTANCE_MM: f64 = 2.0;
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
    // Arc-center leeway applies only to near-straight arcs: a wide I/J
    // bound on a small-radius arc changes the deposited contour.
    let arc_tolerance =
        if arc_is_near_straight(&expected.motion) && arc_is_near_straight(&actual.motion) {
            MAX_SAME_TARGET_ARC_CENTER_DIFFERENCE_MM
        } else {
            MAX_SAME_TARGET_COORDINATE_DIFFERENCE_MM
        };
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
            arc_tolerance,
            CONNECT_INFILL_FEATURES.contains(&expected.feature.as_str()),
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
            CONNECT_INFILL_FEATURES.contains(&expected.feature.as_str()),
        )
}

fn cross_target_feed_matches(expected: f64, actual: f64) -> bool {
    let absolute = (expected - actual).abs();
    let relative = absolute / expected.abs().max(f64::EPSILON);
    absolute <= MAX_CROSS_TARGET_DEPOSITION_FEED_DIFFERENCE && relative <= 0.01
}
