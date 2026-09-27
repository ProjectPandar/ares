use super::fuzzy_skin::{FuzzySkinConfig, FuzzySkinNoiseType};
use super::fuzzy_skin_coherent_noise::coherent_value;
use crate::Point2;

pub(super) fn fuzzify_closed_polyline(
    points: &[Point2],
    config: FuzzySkinConfig,
    layer_id: usize,
    print_z: f64,
    noise_type: FuzzySkinNoiseType,
) -> Vec<Point2> {
    if points.len() < 3 {
        return points.to_vec();
    }
    let min_distance = config.point_distance_mm * 0.75;
    let range = config.point_distance_mm * 0.5;
    let mut distance_left_over = unit_random(layer_id, 0, 0, 0) * (min_distance / 2.0);
    let mut previous = *points.last().unwrap();
    let mut generated = Vec::with_capacity(points.len());
    let mut generated_index = 0usize;

    for (segment_index, current) in points.iter().copied().enumerate() {
        let dx = current.x() - previous.x();
        let dy = current.y() - previous.y();
        let length = (dx * dx + dy * dy).sqrt();
        if length > 0.0 {
            let mut distance = distance_left_over;
            while distance < length {
                let ratio = distance / length;
                let base_x = previous.x() + dx * ratio;
                let base_y = previous.y() + dy * ratio;
                let base = Point2::new(base_x, base_y);
                let signed_noise = signed_noise(
                    noise_type,
                    base,
                    config,
                    (layer_id, print_z, segment_index, generated_index),
                );
                generated.push(Point2::new(
                    base.x() + dy / length * signed_noise * config.thickness_mm,
                    base.y() - dx / length * signed_noise * config.thickness_mm,
                ));
                generated_index += 1;
                distance +=
                    min_distance + unit_random(layer_id, segment_index, generated_index, 2) * range;
            }
            distance_left_over = distance - length;
        }
        previous = current;
    }

    for point in points.iter().rev().skip(1).copied() {
        if generated.len() >= 3 {
            break;
        }
        if !generated.contains(&point) {
            generated.push(point);
        }
    }

    if generated.len() >= 3 {
        generated
    } else {
        points.to_vec()
    }
}

fn signed_noise(
    noise_type: FuzzySkinNoiseType,
    base: Point2,
    config: FuzzySkinConfig,
    sample_context: (usize, f64, usize, usize),
) -> f64 {
    let (layer_id, print_z, segment_index, generated_index) = sample_context;
    match noise_type {
        FuzzySkinNoiseType::Classic => {
            unit_random(layer_id, segment_index, generated_index, 1) * 2.0 - 1.0
        }
        FuzzySkinNoiseType::Perlin
        | FuzzySkinNoiseType::Billow
        | FuzzySkinNoiseType::RidgedMulti
        | FuzzySkinNoiseType::Voronoi => coherent_value(noise_type, base, print_z, config),
        FuzzySkinNoiseType::Ripple => unreachable!("ripple is dispatched separately"),
    }
}

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

pub(super) fn ripple_closed_polyline(
    points: &[Point2],
    config: FuzzySkinConfig,
    layer_id: usize,
) -> Vec<Point2> {
    if points.len() < 3 {
        return points.to_vec();
    }
    // Upstream's input is a scaled-i64 polygon (`FuzzySkin.cpp:159`):
    // `unscale_`d mm values differ from our raw f64 input by up to 0.5µm,
    // so quantize FIRST, then run anchor/perimeter/resample on the same
    // truncated inputs upstream sees.
    let scaled: Vec<(i64, i64)> = points
        .iter()
        .map(|point| (scaled_i64(point.x()), scaled_i64(point.y())))
        .collect();
    let mm_points: Vec<Point2> = scaled
        .iter()
        .map(|point| Point2::new(point.0 as f64 * 1e-6, point.1 as f64 * 1e-6))
        .collect();
    // Same `unscale_(norm(scaled))` order as above (`FuzzySkin.cpp:164-166`).
    let perimeter_mm = scaled
        .iter()
        .zip(scaled.iter().cycle().skip(1).take(scaled.len()))
        .map(|(start, end)| {
            let dx = (end.0 - start.0) as f64;
            let dy = (end.1 - start.1) as f64;
            (dx * dx + dy * dy).sqrt() / SCALING_FACTOR
        })
        .sum::<f64>();
    if perimeter_mm < 1e-6 || config.point_distance_mm < 1e-6 {
        return points.to_vec();
    }

    let anchor_arc_mm = ripple_anchor_arc_mm(&mm_points);
    let phase_shift_rad = ripple_phase_shift_rad(config, layer_id);
    let mut generated = Vec::with_capacity((perimeter_mm / config.point_distance_mm) as usize);
    let mut accumulated_mm = 0.0;

    // `pt = p0 + (seg * t).cast<coord_t>()` truncates the interpolated base
    // point toward zero, and the perp offset is a separately truncated
    // `scale_(disp)` — the double quantization is load-bearing.
    let mut generated_scaled: Vec<(i64, i64)> = Vec::with_capacity(generated.capacity());
    for index in 0..scaled.len() {
        let start = scaled[index];
        let end = scaled[(index + 1) % scaled.len()];
        let dx = (end.0 - start.0) as f64;
        let dy = (end.1 - start.1) as f64;
        // Same Eigen `.norm()` — plain sqrt, not `hypot`.
        let length = (dx * dx + dy * dy).sqrt();
        if length < f64::EPSILON {
            continue;
        }
        let length_mm = length / SCALING_FACTOR;
        let segment_end_mm = accumulated_mm + length_mm;
        let mut sample_mm =
            (accumulated_mm / config.point_distance_mm).ceil() * config.point_distance_mm;
        while sample_mm < segment_end_mm {
            let ratio = (sample_mm - accumulated_mm) / length_mm;
            // `phase_at_anchor` groups `2π + phase_shift` first
            // (`FuzzySkin.cpp:183-185`) — see the scaled variant.
            let phase_at_anchor = std::f64::consts::TAU + phase_shift_rad;
            let displacement = (config.ripples_per_layer as f64
                * std::f64::consts::TAU
                * (sample_mm - anchor_arc_mm)
                / perimeter_mm
                + phase_at_anchor)
                .sin()
                * config.thickness_mm;
            // `p0 + (seg * t).cast<coord_t>()` (`FuzzySkin.cpp:202`):
            // upstream truncates the scaled `seg * t` vector to coord_t
            // BEFORE adding the integer vertex — truncating the sum would
            // differ by one ulp whenever `seg * t` carries a negative
            // fractional component past an integer boundary.
            let base_x = start.0 + (dx * ratio) as i64;
            let base_y = start.1 + (dy * ratio) as i64;
            let off_x = (-(dy / length) * displacement * SCALING_FACTOR) as i64;
            let off_y = ((dx / length) * displacement * SCALING_FACTOR) as i64;
            generated_scaled.push((base_x + off_x, base_y + off_y));
            sample_mm += config.point_distance_mm;
        }
        accumulated_mm = segment_end_mm;
    }

    for point in &generated_scaled {
        generated.push(Point2::new(point.0 as f64 * 1e-6, point.1 as f64 * 1e-6));
    }
    while generated.len() < 3 {
        generated.push(points[points.len() - 2]);
    }
    generated
}

/// Upstream `SCALING_FACTOR` (mm -> coord_t, `libslic3r.h:60`).
const SCALING_FACTOR: f64 = 1e6;

fn scaled_i64(value: f64) -> i64 {
    (value * SCALING_FACTOR) as i64
}

fn ripple_phase_shift_rad(config: FuzzySkinConfig, layer_id: usize) -> f64 {
    if config.ripple_offset_percent == 0.0 {
        return 0.0;
    }
    let period_index = layer_id / config.layers_between_ripple_offset;
    (period_index as f64 * config.ripple_offset_percent / 100.0 * std::f64::consts::TAU)
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

fn unit_random(layer_id: usize, segment_index: usize, generated_index: usize, salt: u64) -> f64 {
    let mut hash = 0xcbf29ce484222325u64;
    for value in [
        layer_id as u64,
        segment_index as u64,
        generated_index as u64,
        salt,
    ] {
        hash ^= value.wrapping_add(0x9e3779b97f4a7c15);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    ((hash >> 11) as f64 + 0.5) / ((1u64 << 53) as f64)
}
