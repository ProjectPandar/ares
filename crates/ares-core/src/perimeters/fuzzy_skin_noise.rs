use super::fuzzy_skin::FuzzySkinConfig;
use crate::Point2;

pub(super) fn ripple_scaled_closed_polyline(
    points: &[crate::geometry::Point],
    config: FuzzySkinConfig,
    layer_id: usize,
    scale: crate::geometry::CoordinateScale,
) -> Vec<crate::geometry::Point> {
    if points.len() < 3 {
        return points.to_vec();
    }
    let scaled_thickness = scale.checked_scale(config.thickness_mm).unwrap();
    let scaled_step = scale.checked_scale(config.point_distance_mm).unwrap();
    let amplitude = scale.unscale(scaled_thickness);
    let step = scale.unscale(scaled_step);
    let mm_points = points
        .iter()
        .map(|point| Point2::new(scale.unscale(point.x()), scale.unscale(point.y())))
        .collect::<Vec<_>>();
    // `FuzzySkin.cpp:164-166`: `perimeter_mm += unscale_(norm(scaled))` —
    // the norm is taken in scaled-i64 space and THEN unscaled per segment;
    // computing the norm on mm-space doubles rounds differently.
    let perimeter = (0..points.len())
        .map(|index| {
            let dx = (points[(index + 1) % points.len()].x() - points[index].x()) as f64;
            let dy = (points[(index + 1) % points.len()].y() - points[index].y()) as f64;
            (dx * dx + dy * dy).sqrt() * scale.factor()
        })
        .sum::<f64>();
    if perimeter < 1e-6 || step < 1e-6 {
        return points.to_vec();
    }
    let anchor = ripple_anchor_arc_mm(&mm_points);
    let phase_shift = ripple_phase_shift_rad(config, layer_id);
    let mut output = Vec::with_capacity((perimeter / step) as usize + points.len() * 2);
    let mut accumulated = 0.0;
    for index in 0..points.len() {
        let start = points[index];
        let end = points[(index + 1) % points.len()];
        let dx = (end.x() - start.x()) as f64;
        let dy = (end.y() - start.y()) as f64;
        // Eigen `.norm()` (`FuzzySkin.cpp:190`) is a plain
        // `sqrt(x*x + y*y)`, not the extended-precision `hypot`.
        let scaled_length = (dx * dx + dy * dy).sqrt();
        if scaled_length < f64::EPSILON {
            continue;
        }
        let length = scaled_length * scale.factor();
        let segment_end = accumulated + length;
        let mut sample = (accumulated / step).ceil() * step;
        while sample < segment_end {
            let ratio = (sample - accumulated) / length;
            // Upstream precomputes `phase_at_anchor = 2π + phase_shift`
            // (`FuzzySkin.cpp:183-185`) — the addition groups as
            // `X + (2π + phase_shift)`, not `(X + 2π) + phase_shift`, which
            // differs by one ulp at sin() input boundaries.
            let phase_at_anchor = std::f64::consts::TAU + phase_shift;
            let phase = config.ripples_per_layer as f64 * std::f64::consts::TAU * (sample - anchor)
                / perimeter
                + phase_at_anchor;
            let displacement = phase.sin() * amplitude / scale.factor();
            let base_x = start.x() + (dx * ratio) as i64;
            let base_y = start.y() + (dy * ratio) as i64;
            output.push(crate::geometry::Point::new(
                base_x + (-dy / scaled_length * displacement) as i64,
                base_y + (dx / scaled_length * displacement) as i64,
            ));
            sample += step;
        }
        accumulated = segment_end;
    }
    while output.len() < 3 {
        output.push(points[points.len() - 2]);
    }
    output
}

fn ripple_phase_shift_rad(config: FuzzySkinConfig, layer_id: usize) -> f64 {
    if config.ripple_offset_percent == 0.0 {
        return 0.0;
    }
    let period_index = layer_id / config.layers_between_ripple_offset;
    (period_index as f64 * (config.ripple_offset_percent / 100.0) * std::f64::consts::TAU)
        % std::f64::consts::TAU
}

fn ripple_anchor_arc_mm(points: &[Point2]) -> f64 {
    closest_arc_mm(points, ripple_anchor_point(points))
}

fn ripple_anchor_point(points: &[Point2]) -> Point2 {
    let mut crossing = None;
    for index in 0..points.len() {
        let start = points[index];
        let end = points[(index + 1) % points.len()];
        if (start.y() <= 0.0 && end.y() >= 0.0) || (start.y() >= 0.0 && end.y() <= 0.0) {
            // `FuzzySkin.cpp:116-118`: `t = |yb-ya| < 1e-9 ? 0 : ya/(ya-yb)`;
            // a segment lying flat on Y=0 yields t=0 → x_cross = xa, not the
            // minimum of the endpoints.
            let t = if (end.y() - start.y()).abs() < 1e-9 {
                0.0
            } else {
                start.y() / (start.y() - end.y())
            };
            let x = start.x() + t.clamp(0.0, 1.0) * (end.x() - start.x());
            if crossing.is_none_or(|point: Point2| x < point.x()) {
                crossing = Some(Point2::new(x, 0.0));
            }
        }
    }
    crossing.unwrap_or_else(|| {
        points
            .iter()
            .copied()
            .min_by(|first, second| first.y().abs().total_cmp(&second.y().abs()))
            .unwrap()
    })
}

fn closest_arc_mm(points: &[Point2], anchor: Point2) -> f64 {
    let mut best_arc_mm = 0.0;
    let mut best_distance_squared = f64::INFINITY;
    let mut accumulated_mm = 0.0;
    for index in 0..points.len() {
        let start = points[index];
        let end = points[(index + 1) % points.len()];
        let dx = end.x() - start.x();
        let dy = end.y() - start.y();
        // Upstream divides by `seg_len * seg_len` where `seg_len` is the
        // square root (`FuzzySkin.cpp:135-150`); `sqrt(x)^2` rounds
        // differently than `dx*dx + dy*dy` at ulp boundaries.
        let seg_len_mm = (dx * dx + dy * dy).sqrt();
        if seg_len_mm > 1e-9 {
            let t = (((anchor.x() - start.x()) * dx + (anchor.y() - start.y()) * dy)
                / (seg_len_mm * seg_len_mm))
                .clamp(0.0, 1.0);
            let projection = Point2::new(start.x() + dx * t, start.y() + dy * t);
            let distance_squared = squared_distance(anchor, projection);
            if distance_squared < best_distance_squared {
                best_distance_squared = distance_squared;
                best_arc_mm = accumulated_mm + t * seg_len_mm;
            }
        }
        accumulated_mm += seg_len_mm;
    }
    best_arc_mm
}

fn squared_distance(first: Point2, second: Point2) -> f64 {
    let dx = first.x() - second.x();
    let dy = first.y() - second.y();
    dx * dx + dy * dy
}
