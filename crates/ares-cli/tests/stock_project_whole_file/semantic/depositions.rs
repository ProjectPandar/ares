use super::parser::Deposition;

#[path = "depositions/cross_target.rs"]
mod cross_target;
#[path = "depositions/leftovers.rs"]
mod leftovers;
#[path = "depositions/matching.rs"]
mod matching;

pub(super) use cross_target::compare_deposition_cross_target;

// Features whose segments connect_infill chains: the same set produces
// different merge boundaries AND segment directions across std::sort
// orderings (FillBase.cpp:1700 arc_length sort instability).
pub(super) const CONNECT_INFILL_FEATURES: &[&str] = &[
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
            if !used[index] && matching::deposition_matches(deposition, candidate) {
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
            dump_depositions("expected", &unmatched_expected);
            dump_depositions("actual", &leftovers);
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
    let extrusion_of = |d: &&Deposition| d.extrusion.parse::<f64>().unwrap_or(0.0);
    let valid = |d: &&Deposition, pool: &[&Deposition], full: &[Deposition]| {
        leftovers::leftover_is_valid(d, pool, full)
    };
    let bad_expected = unmatched_expected
        .iter()
        .filter(|d| !valid(d, &leftovers, actual))
        .count();
    let bad_actual = leftovers
        .iter()
        .filter(|d| !valid(d, &unmatched_expected, expected))
        .count();
    if bad_expected > 0 || bad_actual > 0 {
        for d in unmatched_expected
            .iter()
            .filter(|d| !valid(d, &leftovers, actual))
        {
            eprintln!("  bad expected {d:?}");
        }
        for d in leftovers
            .iter()
            .filter(|d| !valid(d, &unmatched_expected, expected))
        {
            eprintln!("  bad actual {d:?}");
        }
        return Err(format!(
            "layer {layer} deposition 1 differs: {} unmatched expected, \
             {} unmatched actual (first {:?} vs {:?})",
            unmatched_expected.len(),
            leftovers.len(),
            unmatched_expected.first(),
            leftovers.first()
        ));
    }
    // Cluster-level coverage was tried and rejected: real merge
    // boundary instability produces strand tips up to ~1.5mm off every
    // oracle deposition path, so vertex-coverage bounds cannot
    // distinguish mutation from instability. The enforceable
    // discriminator is direction-consistency on shared-vertex pairs
    // (see `paired`).
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
                            .map(&measure)
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

fn dump_depositions(side: &str, list: &[&Deposition]) {
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
}

// Same-target tolerances: sub-µm Coord rounding drifts the last emitted
// decimal; arc-fit centers on near-straight arcs (radius ~287mm) can
// differ by up to ~0.6mm without changing the deposited shape. The wide
// bound applies only when BOTH arcs are provably near-straight.
pub(super) const MAX_SAME_TARGET_COORDINATE_DIFFERENCE_MM: f64 = 0.005;
pub(super) const MAX_SAME_TARGET_EXTRUSION_DIFFERENCE_MM: f64 = 0.001;
pub(super) const MAX_SAME_TARGET_ARC_CENTER_DIFFERENCE_MM: f64 = 0.6;
// Reconstructed arc-midpoint deviation bound; center drift is checked
// separately by the near-straight predicate.
pub(super) const MAX_ARC_CONTOUR_DEVIATION_MM: f64 = 0.05;
pub(super) const MAX_SAME_TARGET_DEPOSITION_COUNT_DIFFERENCE: usize = 64;
// connect_infill reshuffles up to ~74 sparse-infill segments per layer
// on this fixture; pair-bounded, not count-bounded.
pub(super) const MAX_SAME_TARGET_UNMATCHED_COUNT: usize = 96;
// Depositions that quantize to a merge/split boundary only shift a tiny
// connect_infill std::sort instability redistributes sparse-infill
// extrusion across merge boundaries; merged moves also drop the tiny
// overlap between joined segments, so bound the net deposited total
// loosely (measured ~0.06mm/layer on ksr_fdmtest_v4).
const MAX_UNCHAINED_EXTRUSION_MISMATCH_MM: f64 = 0.4;
const MAX_UNCHAINED_EXTRUSION_MISMATCH_RATIO: f64 = 0.05;
