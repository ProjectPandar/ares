//! Length weights after `Polyline3::clip_end`, for adaptive loop flow.

use std::borrow::Cow;

use crate::geometry::CoordinateScale;
use crate::project_slice::perimeters::classic::materialize::{ExtrusionPath, Point3};

pub(super) fn mean(paths: &[ExtrusionPath], end_clip: f64, scale: CoordinateScale) -> Option<f64> {
    let mut distance = end_clip / scale.factor();
    let mut count = paths.len();
    // `ExtrusionLoop::clip_end` consumes complete paths in scaled space
    // before `Polyline3::clip_end` quantizes the surviving endpoint.
    while count > 0 && distance > 0.0 {
        let length = scaled_length(&paths[count - 1], 0.0, scale);
        if length > distance {
            break;
        }
        count -= 1;
        distance -= length;
    }
    let mut weighted = 0.0;
    let mut total = 0.0;
    for (index, path) in paths[..count].iter().enumerate() {
        let clip = if index + 1 == count { distance } else { 0.0 };
        let length = scaled_length(path, clip, scale) * scale.factor();
        weighted += path.mm3_per_mm * length;
        total += length;
    }
    (total > 0.0).then(|| weighted / total)
}

fn scaled_length(path: &ExtrusionPath, end_clip: f64, scale: CoordinateScale) -> f64 {
    let points = if end_clip > 0.0 {
        let mut points = path.polyline.points.clone();
        clip(&mut points, end_clip);
        if !path.polyline.fitting.is_empty() {
            let mut fitting = path.polyline.fitting.clone();
            let mut xy: Vec<_> = points
                .iter()
                .map(|point| (scale.unscale(point.x), scale.unscale(point.y)))
                .collect();
            super::super::arc::clip_fitting_end(&mut xy, &mut fitting, scale);
            if let Some((endpoint, projected)) = points.last_mut().zip(xy.last()) {
                endpoint.x = (projected.0 / scale.factor()).round() as i64;
                endpoint.y = (projected.1 / scale.factor()).round() as i64;
            }
        }
        Cow::Owned(points)
    } else {
        Cow::Borrowed(path.polyline.points.as_slice())
    };
    // `MultiPoint3::length` sums scaled Line3 lengths; unscale once,
    // after summation, as `GCode::extrude_loop` does for the PA mean.
    points
        .windows(2)
        .map(|segment| {
            let dx = (segment[1].x - segment[0].x) as f64;
            let dy = (segment[1].y - segment[0].y) as f64;
            let dz = (segment[1].z - segment[0].z) as f64;
            (dx * dx + dy * dy + dz * dz).sqrt()
        })
        .sum::<f64>()
}

fn clip(points: &mut Vec<Point3>, mut distance: f64) {
    while distance > 0.0 {
        let last = points.pop().expect("clipped path has an endpoint");
        let Some(previous) = points.last() else {
            return;
        };
        let dx = (previous.x - last.x) as f64;
        let dy = (previous.y - last.y) as f64;
        let dz = (previous.z - last.z) as f64;
        let squared = dx * dx + dy * dy + dz * dz;
        if squared > distance * distance {
            let ratio = distance / squared.sqrt();
            points.push(Point3 {
                x: (last.x as f64 + dx * ratio) as i64,
                y: (last.y as f64 + dy * ratio) as i64,
                z: (last.z as f64 + dz * ratio) as i64,
            });
            return;
        }
        distance -= squared.sqrt();
    }
}
