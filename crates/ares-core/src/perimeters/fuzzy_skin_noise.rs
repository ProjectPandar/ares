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
    let perimeter = closed_perimeter_length(&mm_points);
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
        let scaled_length = dx.hypot(dy);
        if scaled_length < f64::EPSILON {
            continue;
        }
        let length = scaled_length * scale.factor();
        let segment_end = accumulated + length;
        let mut sample = (accumulated / step).ceil() * step;
        while sample < segment_end {
            let ratio = (sample - accumulated) / length;
            let phase = config.ripples_per_layer as f64 * std::f64::consts::TAU * (sample - anchor)
                / perimeter
                + std::f64::consts::TAU
                + phase_shift;
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
    let perimeter_mm = closed_perimeter_length(points);
    if perimeter_mm < 1e-6 || config.point_distance_mm < 1e-6 {
        return points.to_vec();
    }

    let anchor_arc_mm = ripple_anchor_arc_mm(points);
    let phase_shift_rad = ripple_phase_shift_rad(config, layer_id);
    let mut generated = Vec::with_capacity((perimeter_mm / config.point_distance_mm) as usize);
    let mut accumulated_mm = 0.0;

    for index in 0..points.len() {
        let start = points[index];
        let end = points[(index + 1) % points.len()];
        let dx = end.x() - start.x();
        let dy = end.y() - start.y();
        let length = dx.hypot(dy);
        if length < f64::EPSILON {
            continue;
        }
        let segment_end_mm = accumulated_mm + length;
        let mut sample_mm =
            (accumulated_mm / config.point_distance_mm).ceil() * config.point_distance_mm;
        while sample_mm < segment_end_mm {
            let ratio = (sample_mm - accumulated_mm) / length;
            let displacement = (config.ripples_per_layer as f64
                * std::f64::consts::TAU
                * (sample_mm - anchor_arc_mm)
                / perimeter_mm
                + std::f64::consts::TAU
                + phase_shift_rad)
                .sin()
                * config.thickness_mm;
            generated.push(Point2::new(
                start.x() + dx * ratio - dy / length * displacement,
                start.y() + dy * ratio + dx / length * displacement,
            ));
            sample_mm += config.point_distance_mm;
        }
        accumulated_mm = segment_end_mm;
    }

    while generated.len() < 3 {
        generated.push(points[points.len() - 2]);
    }
    generated
}

fn ripple_phase_shift_rad(config: FuzzySkinConfig, layer_id: usize) -> f64 {
    if config.ripple_offset_percent == 0.0 {
        return 0.0;
    }
    let period_index = layer_id / config.layers_between_ripple_offset;
    (period_index as f64 * config.ripple_offset_percent / 100.0 * std::f64::consts::TAU)
        % std::f64::consts::TAU
}

fn closed_perimeter_length(points: &[Point2]) -> f64 {
    (0..points.len())
        .map(|index| point_distance(points[index], points[(index + 1) % points.len()]))
        .sum()
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
            let x = if start.y().abs() < 1e-9 && end.y().abs() < 1e-9 {
                start.x().min(end.x())
            } else {
                let t = (start.y() / (start.y() - end.y())).clamp(0.0, 1.0);
                start.x() + t * (end.x() - start.x())
            };
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
        let length_squared = dx * dx + dy * dy;
        if length_squared > 1e-18 {
            let t = (((anchor.x() - start.x()) * dx + (anchor.y() - start.y()) * dy)
                / length_squared)
                .clamp(0.0, 1.0);
            let projection = Point2::new(start.x() + dx * t, start.y() + dy * t);
            let distance_squared = squared_distance(anchor, projection);
            if distance_squared < best_distance_squared {
                best_distance_squared = distance_squared;
                best_arc_mm = accumulated_mm + t * length_squared.sqrt();
            }
        }
        accumulated_mm += length_squared.sqrt();
    }
    best_arc_mm
}

fn point_distance(first: Point2, second: Point2) -> f64 {
    squared_distance(first, second).sqrt()
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
