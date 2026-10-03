use super::fitting;
use crate::{
    geometry::CoordinateScale,
    project_slice::perimeters::classic::{
        chained_loops::ExtrusionLoop, materialize::ExtrusionPath,
    },
};

/// `ExtrusionEntity.cpp:182–225`, `MultiPoint.cpp:609–623`.
pub(super) fn split_at_vertex(
    loop_: &mut ExtrusionLoop,
    target: (i64, i64),
    epsilon: f64,
    scale: CoordinateScale,
) -> bool {
    for path_index in 0..loop_.paths.len() {
        let path = &loop_.paths[path_index];
        let mut best = (0, f64::MAX);
        for (index, point) in path.polyline.points.iter().enumerate() {
            let dx = (point.x - target.0) as f64;
            let dy = (point.y - target.1) as f64;
            let distance = dx * dx + dy * dy;
            if distance < best.1 {
                best = (index, distance);
            }
        }
        if best.1 < epsilon * epsilon {
            rotate(loop_, path_index, best.0, scale);
            return true;
        }
    }
    false
}

fn rotate(loop_: &mut ExtrusionLoop, path_index: usize, index: usize, scale: CoordinateScale) {
    if loop_.paths.len() == 1 {
        let path = &mut loop_.paths[0];
        if index > 0 && index + 1 < path.polyline.points.len() {
            let (prefix, mut suffix) = fitting::split_at_index(&path.polyline, index, scale);
            fitting::append_polyline(&mut suffix, prefix);
            path.polyline = suffix;
        }
        return;
    }
    let mut paths = std::mem::take(&mut loop_.paths);
    let following = paths.split_off(path_index + 1);
    let path = paths.pop().expect("matched path exists");
    let (prefix, suffix) = fitting::split_at_index(&path.polyline, index, scale);
    if suffix.points.len() >= 2 {
        loop_.paths.push(ExtrusionPath {
            polyline: suffix,
            ..path
        });
    }
    loop_.paths.extend(following);
    loop_.paths.extend(paths);
    if prefix.points.len() >= 2 {
        loop_.paths.push(ExtrusionPath {
            polyline: prefix,
            ..path
        });
    }
}
