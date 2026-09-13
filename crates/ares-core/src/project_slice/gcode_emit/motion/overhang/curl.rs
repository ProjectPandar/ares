//! Curled-perimeter tracker (`SupportSpotsGenerator.cpp:141-196`).
//!
//! Each layer's external walls are collected as they are emitted; per
//! segment, the distance to the nearest previous-layer line signed
//! through the lower layer's slices and the endpoint curvature feed
//! `estimate_curled_up_height`, and segments above the tolerance become
//! the layer's curled lines. Layer N's estimation consumes layer N−1's
//! lines with their chained heights, so the tracker rolls over on the
//! first path of each new layer — the same layer chaining as
//! `estimate_malformations`.

mod geometry;

#[cfg(test)]
mod tests;

use crate::geometry::{CoordinateScale, LineDistanceTree};

pub(super) use geometry::artificial_distance;

use geometry::{curvatures, estimate_curled_up_height, point_segment_distance, signed_distance};

const CURLING_TOLERANCE_LIMIT: f32 = 0.1;

/// One curled segment above the tolerance, in millimetres
/// (`SupportSpotsGenerator.cpp:192`).
#[derive(Clone, Copy, Debug)]
pub(super) struct CurledLine {
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    curled_height: f32,
}

#[derive(Clone, Copy, Debug)]
struct ExternalLine {
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    curled_height: f32,
}

#[derive(Default)]
pub(crate) struct CurlTracker {
    layer: Option<usize>,
    previous: Vec<ExternalLine>,
    previous_curled: Vec<CurledLine>,
    current: Vec<ExternalLine>,
    current_curled: Vec<CurledLine>,
}

impl CurlTracker {
    /// The previous layer's curled segments for the artificial distance.
    pub(super) fn previous_curled(&mut self, layer_index: usize) -> &[CurledLine] {
        self.roll_over(layer_index);
        &self.previous_curled
    }

    fn roll_over(&mut self, layer_index: usize) {
        if self.layer != Some(layer_index) {
            self.previous = std::mem::take(&mut self.current);
            self.previous_curled = std::mem::take(&mut self.current_curled);
            self.layer = Some(layer_index);
        }
    }

    /// Collects one outer-wall path (`estimate_malformations` over an
    /// `erExternalPerimeter`): per segment, the distance to the nearest
    /// previous-layer line signed through the lower layer's slices, the
    /// endpoint curvature, and the chained curl height.
    pub(super) fn collect(
        &mut self,
        layer_index: usize,
        points: &[(f64, f64)],
        width: f32,
        height: f32,
        boundary: Option<&LineDistanceTree<'_>>,
        scale: CoordinateScale,
    ) {
        self.roll_over(layer_index);
        if points.len() < 2 {
            return;
        }
        let curvature = curvatures(points);
        for (index, pair) in points.windows(2).enumerate() {
            let (a, b) = (pair[0], pair[1]);
            let middle = ((a.0 + b.0) * 0.5, (a.1 + b.1) * 0.5);
            let (distance, bottom_height) = self.nearest_previous(middle);
            let sign = boundary.map_or(1.0, |tree| {
                if signed_distance(tree, scale, middle) + 0.5 * width < 0.0 {
                    -1.0
                } else {
                    1.0
                }
            });
            let curled = estimate_curled_up_height(
                distance * sign,
                0.5 * (curvature[index] + curvature[index + 1]),
                height,
                width,
                bottom_height,
            );
            self.current.push(ExternalLine {
                x0: a.0,
                y0: a.1,
                x1: b.0,
                y1: b.1,
                curled_height: curled,
            });
            if curled > CURLING_TOLERANCE_LIMIT {
                self.current_curled.push(CurledLine {
                    x0: a.0,
                    y0: a.1,
                    x1: b.0,
                    y1: b.1,
                    curled_height: curled,
                });
            }
        }
    }

    /// Unsigned distance to the nearest previous-layer external line and
    /// that line's chained curl height
    /// (`prev_layer_lines.distance_from_lines_extra<false>`).
    fn nearest_previous(&self, point: (f64, f64)) -> (f32, f32) {
        let mut best = (f32::MAX, 0.0_f32);
        for line in &self.previous {
            let distance = point_segment_distance(point, (line.x0, line.y0), (line.x1, line.y1));
            if distance < best.0 {
                best = (distance, line.curled_height);
            }
        }
        best
    }
}
