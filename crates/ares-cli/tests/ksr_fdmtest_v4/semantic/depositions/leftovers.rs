//! Leftover-deposition validation for connect_infill-unstable layers.
//!
//! `depositions_compare` calls [`leftover_is_valid`] for every
//! unmatched deposition on both sides; each accepted leftover must be
//! explainable as a merge-boundary artifact: same feature, same
//! machine state, comparable flow, and consistent geometry.
use crate::semantic::{feed_matches, model, parser::Deposition, position_matches};

use super::matching;

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
// Non-pivot pairings of leftovers up to ~1mm may be direction-exempt;
// real corner caps (measured up to a full 1mm) run perpendicular to
// their nearest counterpart.
const MAX_DIRECTION_EXEMPT_LENGTH_MM: f64 = 1.0;
// |dot| below this is a perpendicular cap; between it and the aligned
// bound is a rotated strand.
const MAX_CAP_DIRECTION_DOT: f64 = 0.3;
// A vertex shared to ~20µm makes the counterpart a pivot of the same
// junction — direction must hold even for short leftovers (real
// perpendicular caps pair ≥0.04mm off the counterpart's vertices).
const MAX_SHARED_VERTEX_DISTANCE_MM: f64 = 0.02;

fn extrusion_of(d: &Deposition) -> f64 {
    d.extrusion.parse::<f64>().unwrap_or(0.0)
}

fn length_of(d: &Deposition) -> f64 {
    let (Ok(sx), Ok(sy), Ok(ex), Ok(ey)) = (
        d.motion.start.x.parse::<f64>(),
        d.motion.start.y.parse::<f64>(),
        d.motion.end.x.parse::<f64>(),
        d.motion.end.y.parse::<f64>(),
    ) else {
        return f64::NAN;
    };
    ((ex - sx).powi(2) + (ey - sy).powi(2)).sqrt()
}

// Extrusion is proportional to path length at fixed width/height: a
// counterpart that covers the same neighborhood with a wildly
// different extrusion-per-mm is an edited segment, not a merge
// boundary artifact.
fn flow_compatible(d: &Deposition, o: &Deposition) -> bool {
    let (le, lo) = (length_of(d), length_of(o));
    if !le.is_finite() || !lo.is_finite() || le <= 0.0 || lo <= 0.0 {
        return true;
    }
    let (flow_e, flow_o) = (extrusion_of(d) / le, extrusion_of(o) / lo);
    (flow_e - flow_o).abs() <= flow_e.abs().max(flow_o.abs()) * 0.25
}

// Machine state must agree on every paired leftover: a 10x feed on
// the same segment is a real speed divergence, not a merge
// artifact. The arc plane is part of the state: a leftover under
// G18 never pairs with a G17 counterpart.
fn machine_state_matches(d: &Deposition, o: &Deposition) -> bool {
    d.width == o.width
        && feed_matches(d.feed, o.feed)
        && d.acceleration == o.acceleration
        && d.fans == o.fans
        && d.motion.plane == o.motion.plane
}

fn degenerate(d: &Deposition) -> bool {
    extrusion_of(d).abs() <= MAX_LEFTOVER_DEPOSITION_EXTRUSION_MM
}

// Segment containment: `inner` lies along `outer`'s span within tol,
// codirectional — reconciles a tiny legit split against the consumed
// larger piece without letting a reversed fragment qualify.
fn contained_codirectional(inner: &Deposition, outer: &Deposition) -> bool {
    let (Ok(isx), Ok(isy), Ok(iex), Ok(iey)) = (
        inner.motion.start.x.parse::<f64>(),
        inner.motion.start.y.parse::<f64>(),
        inner.motion.end.x.parse::<f64>(),
        inner.motion.end.y.parse::<f64>(),
    ) else {
        return false;
    };
    let (Ok(osx), Ok(osy), Ok(oex), Ok(oey)) = (
        outer.motion.start.x.parse::<f64>(),
        outer.motion.start.y.parse::<f64>(),
        outer.motion.end.x.parse::<f64>(),
        outer.motion.end.y.parse::<f64>(),
    ) else {
        return false;
    };
    let (odx, ody) = (oex - osx, oey - osy);
    let (idx, idy) = (iex - isx, iey - isy);
    let olen2 = odx * odx + ody * ody;
    if olen2 <= 0.0 || idx * odx + idy * ody <= 0.0 {
        return false;
    }
    let project = |px: f64, py: f64| ((px - osx) * odx + (py - osy) * ody) / olen2;
    let lateral = |px: f64, py: f64| {
        let t = project(px, py).clamp(0.0, 1.0);
        let (cx, cy) = (osx + t * odx, osy + t * ody);
        ((px - cx).powi(2) + (py - cy).powi(2)).sqrt()
    };
    project(isx, isy) >= -0.02
        && project(iex, iey) <= 1.02
        && lateral(isx, isy) <= 0.02
        && lateral(iex, iey) <= 0.02
}

fn direction_dot(d: &Deposition, o: &Deposition) -> f64 {
    let dir = |m: &model::MotionRecord| {
        let (Ok(sx), Ok(sy), Ok(ex), Ok(ey)) = (
            m.start.x.parse::<f64>(),
            m.start.y.parse::<f64>(),
            m.end.x.parse::<f64>(),
            m.end.y.parse::<f64>(),
        ) else {
            return None;
        };
        let (dx, dy) = (ex - sx, ey - sy);
        let len = (dx * dx + dy * dy).sqrt();
        (len > 0.0).then_some((dx / len, dy / len))
    };
    let (Some((dx, dy)), Some((ox, oy))) = (dir(&d.motion), dir(&o.motion)) else {
        return 1.0;
    };
    (dx * ox + dy * oy).abs()
}

fn direction_aligned(d: &Deposition, o: &Deposition) -> bool {
    direction_dot(d, o) >= MIN_UNMATCHED_DIRECTION_DOT
}

// Genuine corner caps run perpendicular (|dot| ≤ 0.3 observed
// 0.0–0.17 in the fixture) AND land both ends on the counterpart's
// vertices — a rotated strand in the 0.3–0.81 band is rejected,
// and so is a perpendicular piece whose far end wandered off.
fn direction_gated_ok(d: &Deposition, o: &Deposition) -> bool {
    let dot = direction_dot(d, o);
    if dot >= MIN_UNMATCHED_DIRECTION_DOT {
        return true;
    }
    // A rotated strand keeps its length; a corner cap joins legs
    // of different length, so equal-length perpendicular pairs
    // are not caps.
    let (ld, lo) = (length_of(d), length_of(o));
    if dot > MAX_CAP_DIRECTION_DOT || (ld - lo).abs() <= 0.15 * ld.max(lo) {
        return false;
    }
    let bound = (1.2 * lo + 0.1).min(MAX_UNMATCHED_PAIR_DISTANCE_MM);
    (position_matches(&d.motion.start, &o.motion.start, bound)
        && position_matches(&d.motion.end, &o.motion.end, bound))
        || (position_matches(&d.motion.start, &o.motion.end, bound)
            && position_matches(&d.motion.end, &o.motion.start, bound))
}

fn endpoint_pairs<'a>(
    d: &'a Deposition,
    o: &'a Deposition,
) -> [(&'a model::Position, &'a model::Position); 4] {
    [
        (&d.motion.start, &o.motion.start),
        (&d.motion.start, &o.motion.end),
        (&d.motion.end, &o.motion.start),
        (&d.motion.end, &o.motion.end),
    ]
}

// Spatial pairing: same feature + machine state + flow-compatible,
// and the unmatched counterpart shares a boundary endpoint. When
// the counterpart shares a vertex within ~20µm (a rotated/edited
// segment pivots on a shared endpoint; quantization splits land
// ~1µm off), it must keep the strand direction; perpendicular caps
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
        let gated = shared_vertex || (close && length_of(d) > MAX_DIRECTION_EXEMPT_LENGTH_MM);
        // An arc-vs-segment pair has no comparable tangent: the
        // upstream arc fitter legitimately folds a polyline span
        // into one arc, so the direction gate applies only to
        // like-for-like commands. A true pivot (shared vertex)
        // never produces a perpendicular cap.
        !gated
            || d.motion.command != o.motion.command
            || if shared_vertex {
                direction_aligned(d, o)
            } else {
                direction_gated_ok(d, o)
            }
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
                && if d.motion.command == o.motion.command {
                    length_of(d) <= MAX_LEFTOVER_STRAND_LENGTH_MM
                        && flow_compatible(d, o)
                        && (length_of(d) <= MAX_DIRECTION_EXEMPT_LENGTH_MM
                            && !endpoint_pairs(d, o).iter().any(|(a, b)| {
                                position_matches(a, b, MAX_SHARED_VERTEX_DISTANCE_MM)
                            })
                            || direction_gated_ok(d, o))
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
    // move command, z, and segment length must still look real.
    extrusion_of(d).abs() <= MAX_SLIVER_DEPOSITION_EXTRUSION_MM
        && length_of(d) <= MAX_SLIVER_DEPOSITION_LENGTH_MM
        && full.iter().any(|o| {
            o.feature == d.feature
                && machine_state_matches(d, o)
                && o.motion.command == d.motion.command
                && o.motion.start.z == d.motion.start.z
                && o.motion.end.z == d.motion.end.z
        })
}
