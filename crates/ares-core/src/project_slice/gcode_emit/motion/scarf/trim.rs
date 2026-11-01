//! `ExtrusionEntity.cpp:444-481,516-521`: trim neighbors without moving the seam.

use super::{ExtrusionPath, Point3};
use std::borrow::Cow;

pub(super) fn prepare(
    paths: &[ExtrusionPath],
    slope_length: f64,
    scale: f64,
) -> (Cow<'_, [ExtrusionPath]>, f64) {
    let mut paths = Cow::Borrowed(paths);
    let tolerance = 0.05 / scale;
    trim_front(&mut paths, tolerance);
    trim_back(&mut paths, tolerance);
    let trimmed_length = paths
        .iter()
        .map(|path| {
            path.polyline
                .points
                .windows(2)
                .map(|pair| distance(pair[0], pair[1]))
                .sum::<f64>()
                * scale
        })
        .sum();
    (paths, slope_length.min(trimmed_length))
}

fn trim_front(paths: &mut Cow<'_, [ExtrusionPath]>, tolerance: f64) {
    while !paths.is_empty() {
        let points = &paths[0].polyline.points;
        if points.len() < 2 || !shorter(points[0], points[1], tolerance) {
            break;
        }
        if points.len() > 2 {
            let path = &mut paths.to_mut()[0];
            path.polyline.points.remove(1);
            path.polyline.fitting.clear();
        } else if paths.len() > 1 {
            let seam = points[0];
            let paths = paths.to_mut();
            paths.remove(0);
            paths[0].polyline.points[0] = seam;
            paths[0].polyline.fitting.clear();
        } else {
            break;
        }
    }
}

fn trim_back(paths: &mut Cow<'_, [ExtrusionPath]>, tolerance: f64) {
    while !paths.is_empty() {
        let index = paths.len() - 1;
        let points = &paths[index].polyline.points;
        if points.len() < 2
            || !shorter(
                points[points.len() - 2],
                points[points.len() - 1],
                tolerance,
            )
        {
            break;
        }
        if points.len() > 2 {
            let path = &mut paths.to_mut()[index];
            let vertex = path.polyline.points.len() - 2;
            path.polyline.points.remove(vertex);
            path.polyline.fitting.clear();
        } else if paths.len() > 1 {
            let seam = points[1];
            let paths = paths.to_mut();
            paths.pop();
            let path = &mut paths[index - 1];
            let vertex = path.polyline.points.len() - 1;
            path.polyline.points[vertex] = seam;
            path.polyline.fitting.clear();
        } else {
            break;
        }
    }
}

fn shorter(a: Point3, b: Point3, tolerance: f64) -> bool {
    distance(a, b) < tolerance
}

fn distance(a: Point3, b: Point3) -> f64 {
    let x = (b.x - a.x) as f64;
    let y = (b.y - a.y) as f64;
    let z = (b.z - a.z) as f64;
    (x * x + y * y + z * z).sqrt()
}
