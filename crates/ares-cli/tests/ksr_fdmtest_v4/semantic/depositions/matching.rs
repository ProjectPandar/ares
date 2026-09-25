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
fn arc_geometry(m: &model::MotionRecord) -> Option<(f64, f64, f64, f64, f64)> {
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
    if m.command == "G2" {
        if a1 >= a0 {
            a1 -= std::f64::consts::TAU;
        }
    } else if a1 <= a0 {
        a1 += std::f64::consts::TAU;
    }
    let turns = m
        .turns
        .as_deref()
        .and_then(|t| t.parse::<f64>().ok())
        .unwrap_or(0.0)
        .max(0.0);
    let sweep = (a1 - a0) + turns * std::f64::consts::TAU;
    Some((cx, cy, radius, a0, sweep))
}

// Max distance between the two reconstructed arcs at shared fractions
// of the commanded sweep — endpoints already matched, so mid-sweep
// divergence is what an edited center produces.
fn arc_contour_deviation(expected: &model::MotionRecord, actual: &model::MotionRecord) -> f64 {
    let (Some(ge), Some(ga)) = (arc_geometry(expected), arc_geometry(actual)) else {
        return match (arc_geometry(expected), arc_geometry(actual)) {
            (None, None) => 0.0,
            _ => f64::MAX,
        };
    };
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
