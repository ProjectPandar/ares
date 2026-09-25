//! Geometry helpers for leftover validation: lengths, directions,
//! endpoint pairing, arc sanity, and path-distance coverage.
use crate::semantic::{feed_matches, model, parser::Deposition, position_matches};

use super::matching;
use super::{
    MAX_CAP_DIRECTION_DOT, MAX_CAP_JUNCTION_DISTANCE_MM, MAX_LEFTOVER_DEPOSITION_EXTRUSION_MM,
    MIN_UNMATCHED_DIRECTION_DOT,
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
pub(super) fn flow_compatible(d: &Deposition, o: &Deposition) -> bool {
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
pub(super) fn machine_state_matches(d: &Deposition, o: &Deposition) -> bool {
    d.width == o.width
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
    if dot > MAX_CAP_DIRECTION_DOT || (ld - lo).abs() <= 0.15 * ld.max(lo) {
        return false;
    }
    endpoint_pairs(d, o)
        .iter()
        .any(|(a, b)| position_matches(a, b, MAX_CAP_JUNCTION_DISTANCE_MM))
}

// Radius/chord sanity for arc leftovers: the flattest real arc in
// the fixture is ~287:1; ~320 rejects a near-point-circle rewrite
// (0.09mm chord on a 39.5mm radius).
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
        for t in [0.25f64, 0.5, 0.75] {
            let a = a0 + sweep * t;
            pts.push((cx + r * a.cos(), cy + r * a.sin()));
        }
    }
    let (ax, ay) = (dx(&o.motion.start.x), dx(&o.motion.start.y));
    let (bx, by) = (dx(&o.motion.end.x), dx(&o.motion.end.y));
    let mut opts = vec![(ax, ay), (bx, by)];
    if let Some((cx, cy, r, a0, sweep)) = matching::arc_geometry(&o.motion) {
        for t in [0.25f64, 0.5, 0.75] {
            let a = a0 + sweep * t;
            opts.push((cx + r * a.cos(), cy + r * a.sin()));
        }
    }
    pts.iter()
        .map(|p| {
            opts.iter()
                .map(|q| (p.0 - q.0).powi(2) + (p.1 - q.1).powi(2))
                .fold(f64::MAX, f64::min)
                .sqrt()
        })
        .fold(0.0f64, f64::max)
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
