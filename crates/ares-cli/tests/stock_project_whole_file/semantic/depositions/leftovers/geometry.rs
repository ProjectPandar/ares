//! Geometry helpers for leftover validation: lengths, directions,
//! endpoint pairing, arc sanity, and path-distance coverage.
use crate::semantic::{feed_matches, model, parser::Deposition, position_matches};

use super::matching;
use super::{
    MAX_CAP_DIRECTION_DOT, MAX_CAP_JUNCTION_DISTANCE_MM, MAX_CAP_LENGTH_MM,
    MAX_LEFTOVER_DEPOSITION_EXTRUSION_MM, MIN_UNMATCHED_DIRECTION_DOT,
};

pub(super) fn extrusion_of(d: &Deposition) -> f64 {
    d.extrusion.parse::<f64>().unwrap_or(0.0)
}

pub(super) fn length_of(d: &Deposition) -> f64 {
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

// The path a deposition actually sweeps: arc length for G2/G3
// (radius × sweep), chord for straights — a 0.2mm-chord arc on a
// 40mm radius is a ~125mm deposit, not a sliver.
pub(super) fn swept_length_of(d: &Deposition) -> f64 {
    let chord = length_of(d);
    if d.motion.command != "G2" && d.motion.command != "G3" {
        return chord;
    }
    matching::arc_swept_length(&d.motion).unwrap_or(chord)
}

// Extrusion is proportional to path length at fixed width/height: a
// counterpart that covers the same neighborhood with a wildly
// different extrusion-per-mm is an edited segment, not a merge
// boundary artifact.
pub(super) fn flow_within(d: &Deposition, o: &Deposition, ratio: f64) -> bool {
    let per = |a: &Deposition, b: &Deposition, len: fn(&Deposition) -> f64| {
        let (la, lb) = (len(a), len(b));
        if !la.is_finite() || !lb.is_finite() || la <= 0.0 || lb <= 0.0 {
            return true;
        }
        let (fa, fb) = (extrusion_of(a) / la, extrusion_of(b) / lb);
        (fa - fb).abs() <= fa.abs().max(fb.abs()) * ratio
    };
    // Extrusion tracks CHORD on restitches that swap a few
    // arc-chords (a 349° G3 keeps the E its short chord carried),
    // and tracks SWEPT length on exact arc splits. Either
    // normalization consistent with the fixture is accepted; an
    // extrusion rewrite fails both.
    per(d, o, length_of) || per(d, o, swept_length_of)
}

pub(super) fn flow_compatible(d: &Deposition, o: &Deposition) -> bool {
    flow_within(d, o, 0.25)
}

// Machine state must agree on every paired leftover: a 10x feed on
// the same segment is a real speed divergence, not a merge
// artifact. The arc plane is part of the state: a leftover under
// G18 never pairs with a G17 counterpart.
pub(super) fn machine_state_matches(d: &Deposition, o: &Deposition) -> bool {
    d.motion.start.z == o.motion.start.z
        && d.motion.end.z == o.motion.end.z
        && d.width == o.width
        && feed_matches(d.feed, o.feed)
        && d.acceleration == o.acceleration
        && d.fans == o.fans
        && d.motion.plane == o.motion.plane
}

pub(super) fn degenerate(d: &Deposition) -> bool {
    extrusion_of(d).abs() <= MAX_LEFTOVER_DEPOSITION_EXTRUSION_MM
}

// Segment containment: `inner` lies along `outer`'s span within tol,
// codirectional — reconciles a tiny legit split against the consumed
// larger piece without letting a reversed fragment qualify.
pub(super) fn contained_codirectional(inner: &Deposition, outer: &Deposition) -> bool {
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

pub(super) fn direction_dot(d: &Deposition, o: &Deposition) -> f64 {
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

// Genuine corner caps run perpendicular (|dot| ≤ 0.3 observed
// 0.0–0.17 in the fixture) AND land both ends on the counterpart's
// vertices — a rotated strand in the 0.3–0.81 band is rejected,
// and so is a perpendicular piece whose far end wandered off.
pub(super) fn direction_gated_ok(d: &Deposition, o: &Deposition) -> bool {
    let dot = direction_dot(d, o);
    if dot >= MIN_UNMATCHED_DIRECTION_DOT {
        return true;
    }
    // A rotated strand keeps its length and spreads both endpoints;
    // a corner cap joins legs of different length and lands ONE
    // endpoint at the junction (the other meets the counterpart's
    // flank, which can sit far from its vertices on a long leg).
    let (ld, lo) = (length_of(d), length_of(o));
    // A cap is a short tip meeting a longer leg: the short side must
    // be a tip — a long perpendicular strand is a rotated segment.
    if dot > MAX_CAP_DIRECTION_DOT
        || (ld - lo).abs() <= 0.15 * ld.max(lo)
        || ld.min(lo) > MAX_CAP_LENGTH_MM
    {
        return false;
    }
    endpoint_pairs(d, o)
        .iter()
        .any(|(a, b)| position_matches(a, b, MAX_CAP_JUNCTION_DISTANCE_MM))
}

// Radius/chord sanity for arc leftovers: the flattest real arc in
// the fixture is ~287:1; ~320 rejects a near-point-circle rewrite
// (0.09mm chord on a 39.5mm radius). Swept/chord bounds the other
// shape of rewrite: the fixture's loosest real arc travels ~25mm
// per mm of chord (a 349° refit on a shared vertex), while the
// probed edits sweep ~124mm on a ~0.09mm chord; a 65mm-per-chord rewrite still fails the 30× bound.
pub(super) fn sane_arc(d: &Deposition) -> bool {
    let (Ok(i), Ok(j)) = (
        d.motion.arc_center[0]
            .as_deref()
            .unwrap_or("0")
            .parse::<f64>(),
        d.motion.arc_center[1]
            .as_deref()
            .unwrap_or("0")
            .parse::<f64>(),
    ) else {
        return false;
    };
    let chord = length_of(d).max(1e-9);
    (i * i + j * j).sqrt() / chord <= 320.0
        && matching::arc_swept_length(&d.motion).is_some_and(|swept| swept <= chord * 30.0)
}

// Worst-case distance of `d`'s swept path to `o`'s chord: sample
// the arc midpoint too so a bulged arc does not hide behind its
// endpoints.
pub(super) fn path_distance(d: &Deposition, o: &Deposition) -> f64 {
    let dx = |s: &str| s.parse::<f64>().unwrap_or(0.0);
    let (x1, y1) = (dx(&d.motion.start.x), dx(&d.motion.start.y));
    let (x2, y2) = (dx(&d.motion.end.x), dx(&d.motion.end.y));
    let mut pts = vec![(x1, y1), (x2, y2)];
    if let Some((cx, cy, r, a0, sweep)) = matching::arc_geometry(&d.motion) {
        // Sample a single revolution's worth of points; extra turns
        // retrace the same circle.
        let single = sweep.abs().min(std::f64::consts::TAU);
        let dir = sweep.signum();
        for t in [0.125f64, 0.375, 0.625, 0.875] {
            let a = a0 + dir * single * t;
            pts.push((cx + r * a.cos(), cy + r * a.sin()));
        }
    } else if let Some((ocx, ocy, _, _, _)) = matching::arc_geometry(&o.motion) {
        // A straight's distance to a covering arc peaks either at an
        // endpoint or where the segment comes closest to the arc's
        // CENTER (distance to the circle is |dist-to-center - r|, a
        // convex function whose extremum sits at the projection of
        // the center onto the segment). Sampling fixed fractions
        // would just move the missed extremum.
        let (ux, uy) = (x2 - x1, y2 - y1);
        let len2 = ux * ux + uy * uy;
        if len2 > 0.0 {
            let t = (((ocx - x1) * ux + (ocy - y1) * uy) / len2).clamp(0.0, 1.0);
            pts.push((x1 + ux * t, y1 + uy * t));
            // Off-span points measure min(distance to start,
            // distance to end); that function peaks on the
            // endpoints' perpendicular bisector. A segment whose
            // interior crosses the bisector can hide its worst
            // point between the endpoints and the projected
            // center sample.
            let (ax_, ay_) = (dx(&o.motion.start.x), dx(&o.motion.start.y));
            let (bx_, by_) = (dx(&o.motion.end.x), dx(&o.motion.end.y));
            // Solve d1(t)^2 = d2(t)^2 for t along the segment.
            let t_bis = ((bx_ - ax_) * (ax_ + bx_ - 2.0 * x1)
                + (by_ - ay_) * (ay_ + by_ - 2.0 * y1))
                / (2.0 * ((bx_ - ax_) * ux + (by_ - ay_) * uy));
            if t_bis.is_finite() && (0.0..=1.0).contains(&t_bis) {
                pts.push((x1 + ux * t_bis, y1 + uy * t_bis));
            }
        }
    }
    let (ax, ay) = (dx(&o.motion.start.x), dx(&o.motion.start.y));
    let (bx, by) = (dx(&o.motion.end.x), dx(&o.motion.end.y));
    // Multi-turn arcs alias at sweep fractions: compare the circles
    // directly, the same bound `arc_contour_deviation` uses.
    let turns_of = |m: &model::MotionRecord| {
        m.turns
            .as_deref()
            .and_then(|t| t.parse::<f64>().ok())
            .unwrap_or(0.0)
    };
    if (d.motion.command == "G2" || d.motion.command == "G3")
        && (o.motion.command == "G2" || o.motion.command == "G3")
        && (turns_of(&d.motion) > 0.0 || turns_of(&o.motion) > 0.0)
    {
        return matching::arc_contour_deviation(&d.motion, &o.motion);
    }
    // Distance from a sample point to o's swept path: the arc's
    // circle inside its angular span, or the chord segment.
    let dist_to_path = |p: (f64, f64)| -> f64 {
        let seg_dist = |(px, py): (f64, f64)| -> f64 {
            let (ux, uy) = (bx - ax, by - ay);
            let len2 = ux * ux + uy * uy;
            let t = if len2 > 0.0 {
                (((px - ax) * ux + (py - ay) * uy) / len2).clamp(0.0, 1.0)
            } else {
                0.0
            };
            ((px - ax - t * ux).powi(2) + (py - ay - t * uy).powi(2)).sqrt()
        };
        if let Some((ocx, ocy, or_, oa0, osweep)) = matching::arc_geometry(&o.motion) {
            let ang = (p.1 - ocy).atan2(p.0 - ocx);
            let rel = (ang - oa0).rem_euclid(std::f64::consts::TAU);
            let span = osweep.abs().rem_euclid(std::f64::consts::TAU);
            // Strict span membership: a point past the arc's end must
            // measure its true Euclidean distance to the nearest
            // endpoint — granting tangential slack separately from
            // the radial bound sums the two (a point .52mm past the
            // end plus .30mm radial read as .30mm, though its true
            // distance is .60mm).
            // A covering sweep of a full turn or more includes
            // every angle on the circle; only partial sweeps have a
            // missing interval.
            let on_span = osweep.abs() >= std::f64::consts::TAU
                || if osweep >= 0.0 {
                    rel <= span
                } else {
                    rel >= std::f64::consts::TAU - span
                };
            if on_span {
                return (((p.0 - ocx).powi(2) + (p.1 - ocy).powi(2)).sqrt() - or_).abs();
            }
            // Off the swept span: nearest swept point is an endpoint.
            let (sx, sy) = (ax, ay);
            return ((p.0 - sx).powi(2) + (p.1 - sy).powi(2))
                .sqrt()
                .min(((p.0 - bx).powi(2) + (p.1 - by).powi(2)).sqrt());
        }
        seg_dist(p)
    };
    pts.iter().map(|p| dist_to_path(*p)).fold(0.0f64, f64::max)
}

// Both endpoints of EACH side sit within the junction bound of the
// other's endpoints — the two endpoints of a cap can both land near
// one endpoint of the arc it was cut from, which is an adjacency,
// not a shared chord.
pub(super) fn same_chord_pair(d: &Deposition, o: &Deposition) -> bool {
    let cover = |a: &Deposition, b: &Deposition| {
        [&a.motion.start, &a.motion.end].iter().all(|p| {
            [&b.motion.start, &b.motion.end]
                .iter()
                .any(|q| position_matches(p, q, MAX_CAP_JUNCTION_DISTANCE_MM))
        })
    };
    cover(d, o) && cover(o, d)
}

pub(super) fn endpoint_pairs<'a>(
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
