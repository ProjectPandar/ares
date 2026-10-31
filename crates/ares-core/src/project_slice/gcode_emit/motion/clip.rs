pub(in crate::project_slice::gcode_emit) fn clip_end(points: &mut Vec<(i64, i64)>, distance: f64) {
    let (count, endpoint) = clipped_tail(points.iter().copied(), distance);
    points.truncate(count);
    if let Some(endpoint) = endpoint {
        *points.last_mut().expect("the path has an endpoint") = endpoint;
    }
}

pub(in crate::project_slice::gcode_emit) fn clipped_tail(
    mut points: impl DoubleEndedIterator<Item = (i64, i64)> + ExactSizeIterator,
    distance: f64,
) -> (usize, Option<(i64, i64)>) {
    if distance <= 0.0 {
        return (points.len(), None);
    }
    let Some(mut last) = points.next_back() else {
        return (0, None);
    };
    let mut remaining = distance;
    while let Some(previous) = points.next_back() {
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
            return (points.len() + 2, Some(endpoint));
        }
        remaining -= lsqr.sqrt();
        if remaining <= f64::EPSILON {
            return (points.len() + 1, None);
        }
        last = previous;
    }
    (0, None)
}
