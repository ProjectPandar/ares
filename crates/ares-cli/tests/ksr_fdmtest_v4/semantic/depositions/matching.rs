use super::{
    CONNECT_INFILL_FEATURES, MAX_ARC_CONTOUR_DEVIATION_MM,
    MAX_SAME_TARGET_ARC_CENTER_DIFFERENCE_MM, MAX_SAME_TARGET_COORDINATE_DIFFERENCE_MM,
    MAX_SAME_TARGET_EXTRUSION_DIFFERENCE_MM,
};
use crate::semantic::{
    arc_is_near_straight, feed_matches, model, motion_matches, numeric_matches, parser::Deposition,
};

pub(super) fn deposition_matches(expected: &Deposition, actual: &Deposition) -> bool {
    // Arc equivalence is judged by reconstructed contour deviation,
    // not raw center distance: near-straight arc-fit centers are
    // unstable (real drift ~0.6mm on radius ~287mm), so allow large
    // center deltas only while the sampled arc midpoint stays close.
    let arc_tolerance =
        if arc_is_near_straight(&expected.motion) && arc_is_near_straight(&actual.motion) {
            MAX_SAME_TARGET_ARC_CENTER_DIFFERENCE_MM
        } else {
            MAX_SAME_TARGET_COORDINATE_DIFFERENCE_MM
        };
    let motion_ok = motion_matches(
        &expected.motion,
        &actual.motion,
        MAX_SAME_TARGET_COORDINATE_DIFFERENCE_MM,
        arc_tolerance,
        CONNECT_INFILL_FEATURES.contains(&expected.feature.as_str()),
    ) && arc_contour_deviation(&expected.motion, &actual.motion)
        <= MAX_ARC_CONTOUR_DEVIATION_MM;
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
        && motion_ok
}

// Arc parameterization shared by the deviation sampler: center,
// radius, start angle, and signed sweep from the endpoints.
pub fn arc_geometry(m: &model::MotionRecord) -> Option<(f64, f64, f64, f64, f64)> {
    let [Some(i), Some(j)] = &m.arc_center else {
        return None;
    };
    let (Ok(i), Ok(j), Ok(sx), Ok(sy), Ok(ex), Ok(ey)) = (
        i.parse::<f64>(),
        j.parse::<f64>(),
        m.start.x.parse::<f64>(),
        m.start.y.parse::<f64>(),
        m.end.x.parse::<f64>(),
        m.end.y.parse::<f64>(),
    ) else {
        return None;
    };
    let (cx, cy) = (sx + i, sy + j);
    let radius = (i * i + j * j).sqrt();
    if radius == 0.0 {
        return None;
    }
    let a0 = (sy - cy).atan2(sx - cx);
    let mut a1 = (ey - cy).atan2(ex - cx);
    // G2 sweeps clockwise (decreasing angle); G3 counter-clockwise.
    // Extra turns extend the sweep in the SAME direction.
    let turns = m
        .turns
        .as_deref()
        .and_then(|t| t.parse::<f64>().ok())
        .unwrap_or(0.0)
        .max(0.0);
    if m.command == "G2" {
        if a1 >= a0 {
            a1 -= std::f64::consts::TAU;
        }
        a1 -= turns * std::f64::consts::TAU;
    } else {
        if a1 <= a0 {
            a1 += std::f64::consts::TAU;
        }
        a1 += turns * std::f64::consts::TAU;
    }
    let sweep = a1 - a0;
    Some((cx, cy, radius, a0, sweep))
}

// Max distance between the two reconstructed arcs at shared fractions
// of the commanded sweep — endpoints already matched, so mid-sweep
// divergence is what an edited center produces.
pub fn arc_contour_deviation(expected: &model::MotionRecord, actual: &model::MotionRecord) -> f64 {
    let (Some(ge), Some(ga)) = (arc_geometry(expected), arc_geometry(actual)) else {
        return match (arc_geometry(expected), arc_geometry(actual)) {
            (None, None) => 0.0,
            _ => f64::MAX,
        };
    };
    let turns_of = |m: &model::MotionRecord| {
        m.turns
            .as_deref()
            .and_then(|t| t.parse::<f64>().ok())
            .unwrap_or(0.0)
    };
    if turns_of(expected) > 0.0 || turns_of(actual) > 0.0 {
        // Multi-turn arcs sweep a repeated circle: sweep-fraction
        // sampling aliases whole revolutions, so compare the circles
        // directly — center offset plus radius drift is the contour
        // error. Turns differ ⇒ different arcs entirely.
        if turns_of(expected) != turns_of(actual) {
            return f64::MAX;
        }
        let dc = ((ge.0 - ga.0).powi(2) + (ge.1 - ga.1).powi(2)).sqrt();
        return dc + (ge.2 - ga.2).abs();
    }
    // Single-turn arcs: sample at sweep fractions — endpoints already
    // matched, so mid-sweep divergence is what an edited center
    // produces.
    [0.25, 0.5, 0.75]
        .into_iter()
        .map(|t| {
            let point = |(cx, cy, r, a0, sweep): (f64, f64, f64, f64, f64)| {
                let a = a0 + sweep * t;
                (cx + r * a.cos(), cy + r * a.sin())
            };
            let (ex, ey) = point(ge);
            let (ax, ay) = point(ga);
            ((ex - ax).powi(2) + (ey - ay).powi(2)).sqrt()
        })
        .fold(0.0f64, f64::max)
}

// Inner arc riding on the outer arc's contour: every sampled point of
// the inner sweep lies on the outer circle and inside the outer sweep
// — the arc fitter splits one sweep into a small arc plus continuing
// segments, so a partial arc can be contained in a bigger one.
fn arc_is_contained(inner: (f64, f64, f64, f64, f64), outer: (f64, f64, f64, f64, f64)) -> bool {
    let (icx, icy, ir, ia0, isweep) = inner;
    let (ocx, ocy, or_, oa0, osweep) = outer;
    // Multi-turn arcs (sweep > one revolution) retrace the same
    // circle: containment degenerates to circle equality —
    // sweep-fraction sampling aliases whole turns and the angular
    // span covers everything anyway.
    if isweep.abs() > std::f64::consts::TAU || osweep.abs() > std::f64::consts::TAU {
        return ((icx - ocx).powi(2) + (icy - ocy).powi(2)).sqrt() + (ir - or_).abs() <= 0.05;
    }
    // Radial distance of each inner contour sample to the outer circle.
    let radial_ok = [0.25, 0.5, 0.75].into_iter().all(|t| {
        let a = ia0 + isweep * t;
        let (px, py) = (icx + ir * a.cos(), icy + ir * a.sin());
        let dr = ((px - ocx).powi(2) + (py - ocy).powi(2)).sqrt() - or_;
        dr.abs() <= 0.05
    });
    if !radial_ok {
        return false;
    }
    // Inner end angles must sit inside the outer sweep (mod 2π):
    // CCW outer covers rel ∈ [0, span]; CW outer covers rel ∈
    // [TAU−span, TAU] plus the start boundary rel ≈ 0.
    let span = osweep.abs() + 0.04;
    [0.0, 1.0].into_iter().all(|t| {
        let a = ia0 + isweep * t;
        let rel = (a - oa0).rem_euclid(std::f64::consts::TAU);
        if osweep >= 0.0 {
            rel <= span
        } else {
            rel <= 0.04 || rel >= std::f64::consts::TAU - span
        }
    })
}

// Arc length the command actually deposits: radius × |sweep|.
pub fn arc_swept_length(m: &model::MotionRecord) -> Option<f64> {
    arc_geometry(m).map(|(_, _, r, _, sweep)| r * sweep.abs())
}

// Both records are arcs whose inner contour is contained in the outer's.
pub fn arc_containment(inner: &model::MotionRecord, outer: &model::MotionRecord) -> bool {
    // Different turn counts are different arcs, never containment.
    if inner.turns != outer.turns {
        return false;
    }
    matches!(
        (arc_geometry(inner), arc_geometry(outer)),
        (Some(i), Some(o)) if arc_is_contained(i, o)
    )
}
