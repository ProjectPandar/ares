// `FuzzySkin.cpp:89-156/223-291`: Arachne Ripple uses adjacent segments
// only and deliberately does not repair the sampled loop's closing gap.
use super::{config::Config, junctions::pad};
use crate::{
    arachne::ExtrusionJunction,
    geometry::{CoordinateScale, Point},
};
use std::f64::consts::TAU;

pub(super) fn fuzzify(
    junctions: &mut Vec<ExtrusionJunction>,
    config: Config,
    layer_id: usize,
    scale: CoordinateScale,
) {
    let amplitude = scale.unscale(config.thickness);
    let count = config.base.ripples_per_layer as f64;
    let step = scale.unscale(config.point_distance);
    if count <= 0.0 || step < 1e-6 {
        return;
    }
    let perimeter: f64 = junctions
        .windows(2)
        .map(|pair| {
            let dx = (pair[1].point.x() - pair[0].point.x()) as f64;
            let dy = (pair[1].point.y() - pair[0].point.y()) as f64;
            (dx * dx + dy * dy).sqrt() * scale.factor()
        })
        .sum();
    if perimeter < 1e-6 {
        return;
    }
    let anchor = anchor_arc(junctions, scale);
    let period = layer_id / config.base.layers_between_ripple_offset;
    let shift = (period as f64 * (config.base.ripple_offset_percent / 100.0) * TAU) % TAU;
    let mut output = Vec::with_capacity((perimeter / step) as usize + junctions.len() * 2);
    let mut accumulated = 0.0;
    for pair in junctions.windows(2) {
        let (start, end) = (pair[0], pair[1]);
        let dx = (end.point.x() - start.point.x()) as f64;
        let dy = (end.point.y() - start.point.y()) as f64;
        let length = (dx * dx + dy * dy).sqrt();
        if length < 1e-4 {
            continue;
        }
        let length_mm = length * scale.factor();
        let segment_end = accumulated + length_mm;
        let mut distance = (accumulated / step).ceil() * step;
        while distance < segment_end {
            let ratio = (distance - accumulated) / length_mm;
            let phase = count * TAU * (distance - anchor) / perimeter + (TAU + shift);
            let displacement = phase.sin() * amplitude / scale.factor();
            let point = Point::new(
                start.point.x() + (dx * ratio) as i64,
                start.point.y() + (dy * ratio) as i64,
            );
            output.push(ExtrusionJunction::new(
                Point::new(
                    point.x() + (-dy / length * displacement) as i64,
                    point.y() + (dx / length * displacement) as i64,
                ),
                end.width,
                end.perimeter_index,
            ));
            distance += step;
        }
        accumulated = segment_end;
    }
    pad(&mut output, junctions);
    if output.len() >= 3 {
        *junctions = output;
    }
}

fn anchor_arc(junctions: &[ExtrusionJunction], scale: CoordinateScale) -> f64 {
    let point = |index: usize| {
        let p = junctions[index % junctions.len()].point;
        (scale.unscale(p.x()), scale.unscale(p.y()))
    };
    let mut anchor = (f64::MAX, f64::MAX);
    let mut crossing = false;
    for index in 0..junctions.len() {
        let (a, b) = (point(index), point(index + 1));
        if (a.1 <= 0.0 && b.1 >= 0.0) || (a.1 >= 0.0 && b.1 <= 0.0) {
            let t = if (b.1 - a.1).abs() < 1e-9 {
                0.0
            } else {
                a.1 / (a.1 - b.1)
            };
            let x = a.0 + t.clamp(0.0, 1.0) * (b.0 - a.0);
            if !crossing || x < anchor.0 {
                anchor = (x, 0.0);
                crossing = true;
            }
        }
    }
    if !crossing {
        let mut best_y = f64::MAX;
        for index in 0..junctions.len() {
            let p = point(index);
            if p.1.abs() < best_y {
                best_y = p.1.abs();
                anchor = p;
            }
        }
    }
    let mut best_distance = f64::MAX;
    let mut arc = 0.0;
    let mut accumulated = 0.0;
    for index in 0..junctions.len() {
        let (a, b) = (point(index), point(index + 1));
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let length = (dx * dx + dy * dy).sqrt();
        if length > 1e-9 {
            let t = (((anchor.0 - a.0) * dx + (anchor.1 - a.1) * dy) / (length * length))
                .clamp(0.0, 1.0);
            let (x, y) = (a.0 + dx * t - anchor.0, a.1 + dy * t - anchor.1);
            let distance = x * x + y * y;
            if distance < best_distance {
                best_distance = distance;
                arc = accumulated + t * length;
            }
        }
        accumulated += length;
    }
    arc
}
