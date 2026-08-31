/// `Polyline.cpp::clip_end`: preserve Eigen's `squaredNorm()` then `sqrt()`
/// arithmetic before the truncating `coord_t` cast.
pub(super) fn clip_end(points: &mut Vec<(i64, i64)>, distance: f64) {
    if distance <= 0.0 {
        return;
    }
    let mut remaining = distance;
    while points.len() > 1 {
        let last = points[points.len() - 1];
        let previous = points[points.len() - 2];
        let dx = (previous.0 - last.0) as f64;
        let dy = (previous.1 - last.1) as f64;
        let length_squared = dx * dx + dy * dy;
        if length_squared > remaining * remaining {
            let ratio = remaining / length_squared.sqrt();
            let endpoint = (
                (last.0 as f64 + dx * ratio) as i64,
                (last.1 as f64 + dy * ratio) as i64,
            );
            *points.last_mut().expect("the path has an endpoint") = endpoint;
            return;
        }
        points.pop();
        remaining -= length_squared.sqrt();
        if remaining <= f64::EPSILON {
            return;
        }
    }
    points.clear();
}

#[cfg(test)]
mod tests;
