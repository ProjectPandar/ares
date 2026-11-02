//! Polyline-only `CornerSmoother` and `smooth_polyline_corners`.

mod curve;
mod shapes;
mod vector;

use super::EPSILON;
use crate::geometry::{Point, Polyline};
use vector::V;

pub(super) fn smooth(polyline: &mut Polyline, factor: f64, tolerance: f64) {
    let factor = if factor.is_finite() {
        factor.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let ratio = 0.5 * factor;
    if ratio <= 0.0 || polyline.points().len() < 3 {
        return;
    }
    let mut output = Vec::with_capacity(2 * polyline.points().len());
    let mut window = [V(0.0, 0.0); 3];
    let mut held = 0;
    for point in polyline.points() {
        let point = V(point.x() as f64, point.y() as f64);
        if held == 0 {
            emit(&mut output, point);
            window[held] = point;
            held += 1;
        } else if held > 1 && straight(window[held - 2], window[held - 1], point) {
            window[held - 1] = point;
        } else if held < 3 {
            window[held] = point;
            held += 1;
        } else {
            round_corner(window, ratio, tolerance, &mut output);
            window[0] = window[1];
            window[1] = window[2];
            window[2] = point;
        }
    }
    if held > 2 {
        round_corner(window, ratio, tolerance, &mut output);
    }
    if held > 1 {
        emit(&mut output, window[held - 1]);
    }
    output.dedup();
    *polyline = Polyline::new(output);
}

fn emit(output: &mut Vec<Point>, point: V) {
    output.push(Point::new(
        (point.0 + 0.5).floor() as i64,
        (point.1 + 0.5).floor() as i64,
    ));
}

fn straight(previous: V, vertex: V, next: V) -> bool {
    let incoming = vertex - previous;
    let outgoing = next - vertex;
    let incoming_length = incoming.norm();
    let outgoing_length = outgoing.norm();
    if incoming_length < EPSILON || outgoing_length < EPSILON {
        return true;
    }
    let incoming = incoming / incoming_length;
    let outgoing = outgoing / outgoing_length;
    incoming.dot(outgoing) > 0.0 && incoming.cross(outgoing).abs() < EPSILON
}

fn round_corner(window: [V; 3], ratio: f64, tolerance: f64, output: &mut Vec<Point>) {
    let [previous, corner, next] = window;
    let incoming = corner - previous;
    let outgoing = next - corner;
    let incoming_length = incoming.norm();
    let outgoing_length = outgoing.norm();
    if incoming_length < EPSILON || outgoing_length < EPSILON {
        emit(output, corner);
        return;
    }
    let incoming = incoming / incoming_length;
    let outgoing = outgoing / outgoing_length;
    let cross = incoming.cross(outgoing);
    let cosine = (incoming.dot(outgoing) * 1e12).round() * 1e-12;
    if cross.abs() < EPSILON || cosine < -0.9 {
        emit(output, corner);
        return;
    }
    let reach = ratio * incoming_length.min(outgoing_length);
    if !shapes::vertices(reach, cosine, tolerance, |vertex| {
        emit(
            output,
            corner + (incoming * vertex.0 + outgoing * vertex.1) * reach,
        );
    }) {
        emit(output, corner);
    }
}
