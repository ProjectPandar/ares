//! `CornerSmoother` streaming state and `smooth_polyline_corners` adapter.

mod curve;
mod shapes;
mod vector;

use std::convert::Infallible;

use crate::geometry::{Point, Polyline};
use vector::V;

const EPSILON: f64 = 1.0e-4;

#[derive(Clone, Copy)]
pub(super) struct CornerSmoothing {
    pub(super) factor: f64,
    pub(super) tolerance: f64,
}

pub(super) fn sanitize_factor(factor: f64) -> f64 {
    if factor.is_finite() {
        factor.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// `FillCornerSmoothing.hpp::CornerSmoother`, uncapped and without a filter.
pub(super) struct CornerSmoother {
    ratio: f64,
    tolerance: f64,
    window: [V; 3],
    held: usize,
    corner_points: Vec<V>,
}

impl CornerSmoother {
    pub(super) fn new(smoothing: CornerSmoothing) -> Self {
        Self {
            ratio: 0.5 * sanitize_factor(smoothing.factor),
            tolerance: smoothing.tolerance,
            window: [V(0.0, 0.0); 3],
            held: 0,
            corner_points: Vec::new(),
        }
    }

    pub(super) fn push<E>(
        &mut self,
        [x, y]: [f64; 2],
        emit: &mut impl FnMut([f64; 2]) -> Result<(), E>,
    ) -> Result<(), E> {
        let point = V(x, y);
        if self.held == 0 {
            emit([x, y])?;
            self.window[self.held] = point;
            self.held += 1;
        } else if self.held > 1
            && straight(
                self.window[self.held - 2],
                self.window[self.held - 1],
                point,
            )
        {
            self.window[self.held - 1] = point;
        } else if self.held < 3 {
            self.window[self.held] = point;
            self.held += 1;
        } else {
            self.emit_corner(emit)?;
            self.window[0] = self.window[1];
            self.window[1] = self.window[2];
            self.window[2] = point;
        }
        Ok(())
    }

    pub(super) fn flush<E>(
        &mut self,
        emit: &mut impl FnMut([f64; 2]) -> Result<(), E>,
    ) -> Result<(), E> {
        if self.held > 2 {
            self.emit_corner(emit)?;
        }
        if self.held > 1 {
            let V(x, y) = self.window[self.held - 1];
            emit([x, y])?;
        }
        self.held = 0;
        Ok(())
    }

    fn emit_corner<E>(
        &mut self,
        emit: &mut impl FnMut([f64; 2]) -> Result<(), E>,
    ) -> Result<(), E> {
        self.corner_points.clear();
        round_corner(
            self.window,
            self.ratio,
            self.tolerance,
            &mut self.corner_points,
        );
        for &V(x, y) in &self.corner_points {
            emit([x, y])?;
        }
        Ok(())
    }
}

pub(super) fn smooth(polyline: &mut Polyline, factor: f64, tolerance: f64) {
    let factor = sanitize_factor(factor);
    if 0.5 * factor <= 0.0 || polyline.points().len() < 3 {
        return;
    }
    let mut output = Vec::with_capacity(2 * polyline.points().len());
    let mut emit = |[x, y]: [f64; 2]| -> Result<(), Infallible> {
        output.push(Point::new(
            (x + 0.5).floor() as i64,
            (y + 0.5).floor() as i64,
        ));
        Ok(())
    };
    let mut smoother = CornerSmoother::new(CornerSmoothing { factor, tolerance });
    for point in polyline.points() {
        let Ok(()) = smoother.push([point.x() as f64, point.y() as f64], &mut emit);
    }
    let Ok(()) = smoother.flush(&mut emit);
    output.dedup();
    *polyline = Polyline::new(output);
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

fn round_corner(window: [V; 3], ratio: f64, tolerance: f64, output: &mut Vec<V>) {
    let [previous, corner, next] = window;
    let incoming = corner - previous;
    let outgoing = next - corner;
    let incoming_length = incoming.norm();
    let outgoing_length = outgoing.norm();
    if incoming_length < EPSILON || outgoing_length < EPSILON {
        output.push(corner);
        return;
    }
    let incoming = incoming / incoming_length;
    let outgoing = outgoing / outgoing_length;
    let cross = incoming.cross(outgoing);
    let cosine = (incoming.dot(outgoing) * 1e12).round() * 1e-12;
    if cross.abs() < EPSILON || cosine < -0.9 {
        output.push(corner);
        return;
    }
    let reach = ratio * incoming_length.min(outgoing_length);
    if !shapes::vertices(reach, cosine, tolerance, |vertex| {
        output.push(corner + (incoming * vertex.0 + outgoing * vertex.1) * reach);
    }) {
        output.push(corner);
    }
}
