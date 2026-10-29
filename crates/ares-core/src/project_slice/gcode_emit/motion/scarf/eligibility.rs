//! `ExtrusionEntity.cpp:369-424`, over the post-placement unclipped loop.
use super::MotionOptions;
use crate::project_slice::perimeters::classic::materialize::{
    ExtrusionPath, ExtrusionRole, Point3,
};

pub(super) fn permits(
    paths: &[ExtrusionPath],
    unsupported: Option<f32>,
    options: &MotionOptions,
    scale: f64,
) -> bool {
    if !is_smooth(
        paths,
        options.scarf.angle_threshold * std::f64::consts::PI / 180.0,
        options.nozzle_diameter,
        scale,
    ) {
        return false;
    }
    if options.scarf.overhang_threshold > 0.0 {
        let width = if paths[0].role == ExtrusionRole::ExternalPerimeter {
            options.scarf.outer_line_width
        } else {
            options.scarf.inner_line_width
        };
        // A seam without a previous-layer distancer (the first object slice
        // over a raft: `layer_id() > 0` admits it at GCode.cpp:5780, but
        // SeamPlacer.cpp:1083-1092 only assigns unsupported_dist through a
        // previous layer) reaches GCode.cpp:5786-5790 reading the
        // uninitialized SeamCandidate field (SeamPlacer.hpp:59-70). The owner
        // approves 0.0 for this undefined boundary to match observed Native
        // admission, not as a claim that the C++ value is zero. See
        // docs/architecture/ard-scarf-raft-unset-distance.md.
        let unsupported = unsupported.unwrap_or(0.0);
        return f64::from(unsupported)
            < options.scarf.overhang_threshold * f64::from(0.01_f32) * width;
    }
    true
}

fn is_smooth(paths: &[ExtrusionPath], angle: f64, arm: f64, scale: f64) -> bool {
    // `ExtrusionLoop::polygon:162-172` omits each path's final shared endpoint.
    let points: Vec<_> = paths
        .iter()
        .flat_map(|path| &path.polyline.points[..path.polyline.points.len() - 1])
        .collect();
    let count = points.len();
    let distance = |a: &Point3, b: &Point3| {
        let x = a.x as f64 * scale - b.x as f64 * scale;
        let y = a.y as f64 * scale - b.y as f64 * scale;
        (x * x + y * y).sqrt()
    };
    let mut lengths: Vec<f32> = points
        .windows(2)
        .map(|p| distance(p[0], p[1]) as f32)
        .collect();
    lengths.push(distance(points[0], points[count - 1]).max(0.1) as f32);
    let (mut previous, mut next) = (0, 0);
    let (mut to_previous, mut to_next) = (0.0_f32, 0.0_f32);
    while f64::from(to_previous) < arm {
        previous = (previous + count - 1) % count;
        to_previous += lengths[previous];
    }
    for current in 0..count {
        while f64::from(to_previous - lengths[previous]) > arm {
            to_previous -= lengths[previous];
            previous = (previous + 1) % count;
        }
        while f64::from(to_next) < arm {
            to_next += lengths[next];
            next = (next + 1) % count;
        }
        // Point.hpp:127-133 casts the integer differences before cross/dot atan2.
        let a = (
            (points[previous].x - points[current].x) as f64,
            (points[previous].y - points[current].y) as f64,
        );
        let b = (
            (points[next].x - points[current].x) as f64,
            (points[next].y - points[current].y) as f64,
        );
        let turn = (a.0 * b.1 - a.1 * b.0).atan2(a.0 * b.0 + a.1 * b.1);
        if if turn > 0.0 {
            turn < angle
        } else {
            turn > -angle
        } {
            return false;
        }
        to_previous += lengths[current];
        to_next -= lengths[current];
    }
    true
}
