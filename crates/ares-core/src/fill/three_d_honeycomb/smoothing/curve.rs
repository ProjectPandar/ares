//! `FillCornerSmoothing.cpp:37–156` quintic curves and equal-turn tangents.

use super::super::EPSILON;
use super::vector::V;

type Control = [V; 6];

fn control(reach: f64, outgoing: V) -> Control {
    let spacing = 0.3 * reach;
    [
        V(-reach, 0.0),
        V(-(reach - spacing), 0.0),
        V(-(reach - 2.0 * spacing), 0.0),
        outgoing * (reach - 2.0 * spacing),
        outgoing * (reach - spacing),
        outgoing * reach,
    ]
}

fn derivatives(control: Control, t: f64) -> (V, V) {
    let mut work = control;
    for count in (4..=work.len()).rev() {
        for index in 0..count - 1 {
            work[index] = work[index] * (1.0 - t) + work[index + 1] * t;
        }
    }
    let degree = (work.len() - 1) as f64;
    (
        ((work[1] - work[0]) * (1.0 - t) + (work[2] - work[1]) * t) * degree,
        (work[2] - work[1] * 2.0 + work[0]) * (degree * (degree - 1.0)),
    )
}

fn point(control: Control, t: f64) -> V {
    let mut work = control;
    for count in (2..=work.len()).rev() {
        for index in 0..count - 1 {
            work[index] = work[index] * (1.0 - t) + work[index + 1] * t;
        }
    }
    work[0]
}

pub(super) fn peak_curvature(cosine: f64) -> f64 {
    let turn = cosine.clamp(-1.0, 1.0).acos();
    let control = control(1.0, V(cosine, turn.sin()));
    let mut peak: f64 = 0.0;
    for index in 0..=256 {
        let (first, second) = derivatives(control, f64::from(index) / 256.0);
        let speed = first.norm();
        if speed > EPSILON {
            peak = peak.max(first.cross(second).abs() / (speed * speed * speed));
        }
    }
    peak
}

pub(super) struct UnitCorner {
    pub(super) vertices: Vec<V>,
    pub(super) deviation: f64,
}

pub(super) fn tangent_polygon(cosine: f64, count: usize) -> UnitCorner {
    let turn = cosine.clamp(-1.0, 1.0).acos();
    let sine = turn.sin();
    let outgoing = V(cosine, sine);
    let control = control(1.0, outgoing);
    let mut tangents = Vec::with_capacity(2 * count + 1);
    tangents.push((V(-1.0, 0.0), V(1.0, 0.0)));
    let mut from = 0.0;
    for index in 1..2 * count {
        let angle = 0.5 * turn * index as f64 / count as f64;
        let (t, low) = tangent_parameter(control, angle, from);
        from = low;
        tangents.push((point(control, t), derivatives(control, t).0.normalized()));
    }
    tangents.push((outgoing, outgoing));
    let mut vertices = Vec::with_capacity(count);
    let mut deviation: f64 = 0.0;
    for index in 0..count {
        let (point, direction) = tangents[2 * index];
        let (next_point, next_direction) = tangents[2 * index + 2];
        let vertex = point
            + direction
                * ((next_point - point).cross(next_direction) / direction.cross(next_direction));
        deviation = deviation.max((vertex - tangents[2 * index + 1].0).norm());
        let along = vertex.1 / sine;
        vertices.push(V(vertex.0 - along * cosine, along));
    }
    UnitCorner {
        vertices,
        deviation,
    }
}

fn tangent_parameter(control: Control, angle: f64, from: f64) -> (f64, f64) {
    let mut low = from;
    let mut high = 1.0;
    let mut t = 0.5 * (low + high);
    let mut iteration = 0;
    while iteration < 64 && high - low > 1e-15 {
        let (first, second) = derivatives(control, t);
        let error = first.1.atan2(first.0) - angle;
        if error.abs() < 1e-14 {
            break;
        }
        if error < 0.0 {
            low = t;
        } else {
            high = t;
        }
        let rate = first.cross(second) / first.squared_norm();
        let next = if rate > 0.0 { t - error / rate } else { low };
        t = if next > low && next < high {
            next
        } else {
            0.5 * (low + high)
        };
        iteration += 1;
    }
    (t, low)
}
