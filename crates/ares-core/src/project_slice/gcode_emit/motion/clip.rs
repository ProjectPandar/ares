pub(in crate::project_slice::gcode_emit) fn clip_end(points: &mut Vec<(i64, i64)>, distance: f64) {
    if distance <= 0.0 {
        return;
    }
    let mut remaining = distance;
    while points.len() > 1 {
        let last = points[points.len() - 1];
        let previous = points[points.len() - 2];
        // Upstream `Polyline::clip_end` (`Polyline.cpp:52-72`): the squared
        // comparison and the sqrt division order match exactly; the final
        // `.cast<coord_t>()` on the Vec2d is Eigen's per-component
        // `static_cast<coord_t>` — truncation toward zero, NOT round-half-up.
        let vx = (previous.0 - last.0) as f64;
        let vy = (previous.1 - last.1) as f64;
        let lsqr = vx * vx + vy * vy;
        if lsqr > remaining * remaining {
            let factor = remaining / lsqr.sqrt();
            let endpoint = (
                (last.0 as f64 + vx * factor) as i64,
                (last.1 as f64 + vy * factor) as i64,
            );
            *points.last_mut().expect("the path has an endpoint") = endpoint;
            return;
        }
        points.pop();
        remaining -= lsqr.sqrt();
        if remaining <= f64::EPSILON {
            return;
        }
    }
    points.clear();
}
#[cfg(test)]
mod tests;
