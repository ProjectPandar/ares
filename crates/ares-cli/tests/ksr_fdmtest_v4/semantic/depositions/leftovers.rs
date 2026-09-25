//! Leftover-deposition validation for connect_infill-unstable layers.
//!
//! `depositions_compare` calls [`leftover_is_valid`] for every
//! unmatched deposition on both sides; each accepted leftover must be
//! explainable as a merge-boundary artifact: same feature, same
//! machine state, comparable flow, and consistent geometry.
use crate::semantic::{parser::Deposition, position_matches};

#[path = "leftovers/geometry.rs"]
mod geometry;

use super::matching;
use geometry::*;

// Depositions that quantize to a merge/split boundary only shift a tiny
// extrusion amount; a degenerate leftover must still sit on the same
// endpoint as a counterpart of the same feature.
const MAX_LEFTOVER_DEPOSITION_EXTRUSION_MM: f64 = 0.02;
const MAX_DEGENERATE_PAIR_DISTANCE_MM: f64 = 0.01;
// Tiny cap/split slivers need no positional counterpart: upstream
// merge instability emits ~0.005mm-extrusion tips unpaired. Machine
// state, command, z and a real deposition feed of the other side all
// still apply, and the sliver stays geometrically tiny.
const MAX_SLIVER_DEPOSITION_EXTRUSION_MM: f64 = 0.008;
const MAX_SLIVER_DEPOSITION_LENGTH_MM: f64 = 1.0;
// Stranded boundary tips are short pieces of the same hatch row.
const MAX_LEFTOVER_STRAND_LENGTH_MM: f64 = 1.0;
// connect_infill merge boundaries shift segments locally; an unmatched
// deposition pairs only with a counterpart covering the same
// neighborhood.
const MAX_UNMATCHED_PAIR_DISTANCE_MM: f64 = 2.0;
// ~36° cone: the shallowest genuine arc-to-segment connector pair in
// the reference fixture sits at |dot| ≈ 0.82 (chord of a small arc
// versus its straight counterpart). Rotating a segment on a shared
// vertex beyond ~35° is a mutation, not a merge boundary.
const MIN_UNMATCHED_DIRECTION_DOT: f64 = 0.81;
// A counterpart within ~1mm is the same merge neighborhood: a pivot
// or small translation cannot rotate the segment more than ~45° there.
// Corner caps genuinely pair across a 1–2mm gap.
const MAX_DIRECTIONAL_PAIR_DISTANCE_MM: f64 = 1.0;
// Beyond ~5mm a leftover is a strand, not a cap — direction is
// always enforced.
const MAX_LONG_LEFTOVER_LENGTH_MM: f64 = 5.0;
// |dot| below this is a perpendicular cap; between it and the aligned
// bound is a rotated strand.
const MAX_CAP_DIRECTION_DOT: f64 = 0.3;
// Cap junction: one endpoint of a perpendicular pair lands within
// ~0.7mm (up to ~1.1mm at split boundaries) of a counterpart vertex.
const MAX_CAP_JUNCTION_DISTANCE_MM: f64 = 1.2;
// A cap joins a short tip (≤2mm) to a longer leg; longer
// perpendicular strands are rotated segments, not caps.
const MAX_CAP_LENGTH_MM: f64 = 2.0;
// Coverage pairs redraw a strand loosely: flow may drift ~1.35× in
// a real restitch, but an extrusion rewrite (~2×+) fails.
const MAX_COVERED_FLOW_RATIO: f64 = 0.3;
// Restitched cells redraw polylines loosely: real leftovers sit
// ~0.2mm off the redrawn counterpart's path.
const MAX_LEFTOVER_PATH_DISTANCE_MM: f64 = 0.55;
// Same-command arc leftovers redrawing the same chord still agree
// loosely: chained refits deviate ~1.5mm at mid-sweep; a center
// rewrite (r5→r30 on a 10mm chord) deviates ~4.6mm.
const MAX_SAME_COMMAND_ARC_DEVIATION_MM: f64 = 2.0;
// A vertex shared to ~20µm makes the counterpart a pivot of the same
// junction — direction must hold even for short leftovers (real
// perpendicular caps pair ≥0.04mm off the counterpart's vertices).
const MAX_SHARED_VERTEX_DISTANCE_MM: f64 = 0.02;

// are the only direction-exempt short leftover.
fn paired(d: &Deposition, pool: &[&Deposition]) -> bool {
    pool.iter().any(|o| {
        if o.feature != d.feature || !machine_state_matches(d, o) || !flow_compatible(d, o) {
            return false;
        }
        let pairs = endpoint_pairs(d, o);
        let near = pairs
            .iter()
            .any(|(a, b)| position_matches(a, b, MAX_UNMATCHED_PAIR_DISTANCE_MM));
        if !near {
            return false;
        }
        let shared_vertex = pairs
            .iter()
            .any(|(a, b)| position_matches(a, b, MAX_SHARED_VERTEX_DISTANCE_MM));
        let close = pairs
            .iter()
            .any(|(a, b)| position_matches(a, b, MAX_DIRECTIONAL_PAIR_DISTANCE_MM));
        // The direction gate applies to every close pairing (a pivot
        // or a near translation cannot turn a real segment), and to
        // long strands at any distance: a multi-mm leftover is never
        // a merge cap. Short non-close pairings are cross-gap
        // connectors and stay direction-free.
        let gated = shared_vertex || close || length_of(d) > MAX_LONG_LEFTOVER_LENGTH_MM;
        // An arc-vs-segment pair has no comparable tangent: the
        // upstream arc fitter legitimately folds a polyline span
        // into one arc, so the direction gate applies only to
        // like-for-like commands. Any arc in the pair instead needs
        // a sane radius/sweep AND a shared endpoint junction — a
        // chord direction says nothing about the swept curve.
        let arc_pair = (d.motion.command == "G2" || d.motion.command == "G3")
            || (o.motion.command == "G2" || o.motion.command == "G3");
        if arc_pair {
            let arc_sane = |s: &Deposition| {
                (s.motion.command != "G2" && s.motion.command != "G3") || sane_arc(s)
            };
            // The junction between a refit arc and its split
            // counterpart wanders ~0.4mm in the fixture — the arc
            // sanity bound, not endpoint coincidence, is the guard.
            // Same-command arcs redrawing the SAME chord (both
            // endpoints shared) must keep roughly the same contour —
            // real chained refits deviate ~1.5mm, a center rewrite
            // deviates ~4.6mm. Arcs that share only ONE endpoint are
            // adjacent segments of one contour (a G3↔G1 restitch or
            // a split arc pair) where divergence is the rule; a
            // command swap (G2↔G3 on one chord) is the unstable
            // signature and skips this too.
            // Both endpoints of EACH side must sit within the
            // junction bound of the other's endpoints — counting raw
            // close pairs misreads a cap whose two ends both land
            // near one endpoint of the big arc as a shared chord.
            let same_chord = same_chord_pair(d, o);
            let same_contour = d.motion.command != o.motion.command
                || !same_chord
                || matching::arc_contour_deviation(&d.motion, &o.motion)
                    <= MAX_SAME_COMMAND_ARC_DEVIATION_MM;
            return arc_sane(d)
                && arc_sane(o)
                && d.motion.turns == o.motion.turns
                && same_contour
                && flow_within(d, o, MAX_COVERED_FLOW_RATIO)
                && pairs
                    .iter()
                    .any(|(a, b)| position_matches(a, b, MAX_CAP_JUNCTION_DISTANCE_MM));
        }
        !gated || direction_gated_ok(d, o)
    })
}

fn unchained(d: &Deposition) -> bool {
    super::CONNECT_INFILL_FEATURES.contains(&d.feature.as_str())
}

// One unmatched deposition is a legitimate merge-boundary artifact
// when any of the reconciliation paths accepts it.
pub(super) fn leftover_is_valid(d: &Deposition, pool: &[&Deposition], full: &[Deposition]) -> bool {
    if unchained(d) && paired(d, pool) {
        return true;
    }
    // A non-degenerate arc whose contour rides inside a same-feature
    // outer arc on the other side is a partial restitch of that
    // sweep, not a new segment.
    if unchained(d)
        && (d.motion.command == "G2" || d.motion.command == "G3")
        && full.iter().any(|o| {
            o.feature == d.feature
                && machine_state_matches(d, o)
                && o.motion.start.z == d.motion.start.z
                && o.motion.end.z == d.motion.end.z
                && flow_within(d, o, MAX_COVERED_FLOW_RATIO)
                && matching::arc_containment(&d.motion, &o.motion)
        })
    {
        return true;
    }
    // Both sides leave leftovers when upstream restitches a hatch
    // boundary: a short tip segment can end up stranded on one side
    // while its counterpart survives on the other.
    if unchained(d)
        && pool.iter().any(|o| {
            o.feature == d.feature
                && machine_state_matches(d, o)
                && endpoint_pairs(d, o)
                    .iter()
                    .any(|(a, b)| position_matches(a, b, MAX_UNMATCHED_PAIR_DISTANCE_MM))
                && if (d.motion.command == "G2" || d.motion.command == "G3")
                    || (o.motion.command == "G2" || o.motion.command == "G3")
                {
                    // Arc restitch: the refit curve is the unstable
                    // part — the oracle emits an 18° G2 where ares
                    // emits a 349° G3 landing on the same chain
                    // vertex — so sharing ONE endpoint junction plus
                    // same feature/state is the stable signature. A
                    // radius/chord and swept/chord sanity bound still
                    // rejects point-circle rewrites on either side;
                    // a non-arc counterpart needs no arc check.
                    let arc_sane = |s: &Deposition| {
                        (s.motion.command != "G2" && s.motion.command != "G3") || sane_arc(s)
                    };
                    arc_sane(d)
                        && arc_sane(o)
                        && d.motion.turns == o.motion.turns
                        && flow_within(d, o, MAX_COVERED_FLOW_RATIO)
                        && (d.motion.command != o.motion.command
                            || !same_chord_pair(d, o)
                            || matching::arc_contour_deviation(&d.motion, &o.motion)
                                <= MAX_SAME_COMMAND_ARC_DEVIATION_MM)
                        && endpoint_pairs(d, o)
                            .iter()
                            .any(|(a, b)| position_matches(a, b, MAX_CAP_JUNCTION_DISTANCE_MM))
                } else if d.motion.command == o.motion.command {
                    // Same-command restitched tips stay short,
                    // flow-consistent, and direction-consistent — a
                    // rotated strand is not a restitch artifact.
                    length_of(d) <= MAX_LEFTOVER_STRAND_LENGTH_MM
                        && flow_compatible(d, o)
                        && direction_gated_ok(d, o)
                } else {
                    // Arc-vs-polyline restitching swaps a G2/G3 for
                    // the straight covering the same chord: endpoints
                    // must coincide closely and extrusion is not
                    // comparable (the arc replaces a longer span).
                    position_matches(
                        &d.motion.start,
                        &o.motion.start,
                        MAX_DEGENERATE_PAIR_DISTANCE_MM,
                    ) && position_matches(
                        &d.motion.end,
                        &o.motion.end,
                        MAX_DEGENERATE_PAIR_DISTANCE_MM,
                    ) || position_matches(
                        &d.motion.start,
                        &o.motion.end,
                        MAX_DEGENERATE_PAIR_DISTANCE_MM,
                    ) && position_matches(
                        &d.motion.end,
                        &o.motion.start,
                        MAX_DEGENERATE_PAIR_DISTANCE_MM,
                    )
                }
        })
    {
        return true;
    }
    if degenerate(d)
        && pool.iter().any(|o| {
            o.feature == d.feature
                && machine_state_matches(d, o)
                && flow_compatible(d, o)
                && o.motion.command == d.motion.command
                && o.motion.arc_center == d.motion.arc_center
                && o.motion.turns == d.motion.turns
                && position_matches(
                    &d.motion.start,
                    &o.motion.start,
                    MAX_DEGENERATE_PAIR_DISTANCE_MM,
                )
                && position_matches(
                    &d.motion.end,
                    &o.motion.end,
                    MAX_DEGENERATE_PAIR_DISTANCE_MM,
                )
        })
    {
        return true;
    }
    // Degenerate split: contained along a same-command same-state
    // deposition on the other side. A straight outer covers a
    // chord-equal straight inner; an inner arc needs contour
    // containment in a bigger arc (the fitter folds polyline spans
    // into arcs, so endpoint containment alone lets a curved
    // leftover hide inside a straight chord).
    if degenerate(d)
        && full.iter().any(|o| {
            o.feature == d.feature
                && machine_state_matches(d, o)
                && o.motion.command == d.motion.command
                && o.motion.plane == d.motion.plane
                && o.motion.start.z == d.motion.start.z
                && o.motion.end.z == d.motion.end.z
                && (matching::arc_containment(&d.motion, &o.motion)
                    || contained_codirectional(d, o)
                        && d.motion.command != "G2"
                        && d.motion.command != "G3")
        })
    {
        return true;
    }
    // Sub-micron-quantization slivers: merge boundaries also emit
    // tiny cap pieces (measured 0.004mm extrusion on a 0.12mm tip)
    // with no counterpart at all. Below the sliver budget the
    // extrusion is physically negligible, but machine state, the
    // move command, z, and swept length must still look real — and
    // an arc sliver must fold into a same-state arc on the other
    // side (a 40mm-radius rewrite of a 0.05mm arc is an edit, not a
    // sliver).
    // Coverage: upstream can redraw a whole mini-polyline inside one
    // connect cell (expected 4 strands vs actual 2 diagonals through
    // the same region). A leftover is covered when every sampled
    // point of its swept path stays close to a same-feature
    // same-state leftover — a strand rotated in place sweeps far
    // from its original chord, and so does a rewritten arc.
    if unchained(d)
        && full.iter().any(|o| {
            o.feature == d.feature
                && machine_state_matches(d, o)
                && flow_within(d, o, MAX_COVERED_FLOW_RATIO)
                && path_distance(d, o) <= MAX_LEFTOVER_PATH_DISTANCE_MM
        })
    {
        return true;
    }
    extrusion_of(d).abs() <= MAX_SLIVER_DEPOSITION_EXTRUSION_MM
        && swept_length_of(d) <= MAX_SLIVER_DEPOSITION_LENGTH_MM
        && full.iter().any(|o| {
            o.feature == d.feature
                && machine_state_matches(d, o)
                && o.motion.command == d.motion.command
                && o.motion.start.z == d.motion.start.z
                && o.motion.end.z == d.motion.end.z
                && (d.motion.command != "G2" && d.motion.command != "G3"
                    || matching::arc_containment(&d.motion, &o.motion))
        })
}
