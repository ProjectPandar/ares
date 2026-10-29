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

use super::super::LayerGeometry;
use crate::geometry::{CoordinateScale, FloatLineDistanceTree, LineDistanceTree};

pub(super) use geometry::artificial_distance;

use geometry::{curvatures, estimate_curled_up_height, signed_distance};

const CURLING_TOLERANCE_LIMIT: f32 = 0.1;

/// One curled segment above the tolerance, stored in scaled-int
/// coordinates (`Point::new_scale` — `ExtrusionProcessor.hpp:381`-
/// `416` compares against the scaled `CurledLine` endpoints).
#[derive(Clone, Copy, Debug)]
pub(super) struct CurledLine {
    x0: i64,
    y0: i64,
    x1: i64,
    y1: i64,
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

#[derive(Clone, Copy)]
struct AnnotatedPoint {
    position: (f64, f64),
    distance: f32,
}

#[derive(Default)]
pub(crate) struct CurlTracker {
    layer: Option<usize>,
    previous: Vec<ExternalLine>,
    previous_tree: FloatLineDistanceTree<f32>,
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
            // `SupportSpotsGenerator.cpp:151,195`: construct the indexed
            // previous-layer distancer once, preserving collected line order.
            self.previous_tree = FloatLineDistanceTree::new(
                self.previous
                    .iter()
                    .map(|line| {
                        [
                            [line.x0 as f32, line.y0 as f32],
                            [line.x1 as f32, line.y1 as f32],
                        ]
                    })
                    .collect(),
            );
            self.previous_curled = std::mem::take(&mut self.current_curled);
            self.layer = Some(layer_index);
        }
    }

    /// Collects one whole external-perimeter ENTITY — an
    /// `ExtrusionLoop`'s or `ExtrusionMultiPath`'s concatenated sub-path
    /// points (`ExtrusionEntity.hpp:507-512,431-435`). Doing this at
    /// entity level, before seam placement, gives `collect` the closed
    /// loop upstream feeds `estimate_malformations`: `looped` sees
    /// `front == back` and the 8 mm curvature `walk` wraps across the
    /// seam instead of truncating at emitted-path fragment ends. Returns
    /// early for non-external roles (`erExternalPerimeter` filter,
    /// `SupportSpotsGenerator.cpp:159-160`).
    pub(in crate::project_slice::gcode_emit) fn collect_entity(
        &mut self,
        layer_index: usize,
        paths: &[crate::project_slice::perimeters::classic::materialize::ExtrusionPath],
        geometry: &LayerGeometry<'_>,
    ) {
        use crate::project_slice::perimeters::classic::materialize::ExtrusionRole;
        if !paths
            .first()
            .is_some_and(|path| path.role == ExtrusionRole::ExternalPerimeter)
        {
            return;
        }
        let points: Vec<(f64, f64)> = paths
            .iter()
            .flat_map(|path| {
                path.polyline.points.iter().map(|point| {
                    (
                        geometry.scale.unscale(point.x),
                        geometry.scale.unscale(point.y),
                    )
                })
            })
            .collect();
        self.collect(
            layer_index,
            &points,
            (paths[0].width, paths[0].height),
            geometry,
        );
    }

    /// Collects one outer-wall path (`estimate_malformations` over an
    /// `erExternalPerimeter`): the entity points are first annotated
    /// (`annotate`), then per segment the distance to the nearest
    /// previous-layer line signed through the lower layer's slices, the
    /// endpoint curvature, and the chained curl height.
    pub(in crate::project_slice::gcode_emit) fn collect(
        &mut self,
        layer_index: usize,
        points: &[(f64, f64)],
        (width, height): (f32, f32),
        geometry: &LayerGeometry<'_>,
    ) {
        let boundary = geometry.previous_layer_boundary;
        let scale = geometry.scale;
        self.roll_over(layer_index);
        if points.len() < 2 {
            return;
        }
        let annotated = self.annotate(points, width);
        let sequence: Vec<(f64, f64)> = annotated.iter().map(|point| point.position).collect();
        let curvature = curvatures(&sequence);
        // `SupportSpotsGenerator.cpp:166-170`: `a = i > 0 ? annotated[i - 1]
        // : annotated[i]` — the i == 0 pair is DEGENERATE (a == b == the
        // first point) and still produces a line with the corner curvature
        // (the oracle's first curled line (p0, p0) with the π/4 corner
        // curvature); `windows(2)` alone would skip it.
        for index in 0..sequence.len() {
            let previous = index.saturating_sub(1);
            let (a, b) = (sequence[previous], sequence[index]);
            // Upstream stores `ExtrusionLine.a/b` as `Vec2f` — both
            // endpoints and the midpoint are rounded to f32
            // (`SupportSpotsGenerator.cpp:170-174`).
            let a = (a.0 as f32 as f64, a.1 as f32 as f64);
            let b = (b.0 as f32 as f64, b.1 as f32 as f64);
            let middle = (
                f64::from((a.0 as f32 + b.0 as f32) * 0.5),
                f64::from((a.1 as f32 + b.1 as f32) * 0.5),
            );
            let (distance, bottom_height) = self.nearest_previous(middle);
            let sign = boundary.map_or(1.0_f64, |tree| boundary_sign(tree, scale, middle, width));
            // Upstream `ExtrusionLine` queries are float-precision —
            // `middle_distance` is already narrowed to float by the
            // distancer, and the boundary `sign` check happens in f64
            // on the `lslices` Linef boundary. The combined distance is
            // narrowed to float at the `estimate_curled_up_height` call
            // (`SupportSpotsGenerator.cpp:181-183`).
            let curled = estimate_curled_up_height(
                (f64::from(distance) * sign) as f32,
                0.5 * (curvature[previous] + curvature[index]),
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
                    x0: super::scale_trunc(a.0, scale),
                    y0: super::scale_trunc(a.1, scale),
                    x1: super::scale_trunc(b.0, scale),
                    y1: super::scale_trunc(b.1, scale),
                    curled_height: curled,
                });
            }
        }
    }

    /// `estimate_points_properties<true, true, false, false>`
    /// (`ExtrusionProcessor.hpp:36-215`) at `boundary_offset = 0`:
    /// unsigned previous-layer distances, intersection points where
    /// consecutive distances straddle `EPSILON` (crossing the previous
    /// layer's lines, `distance = 0`, `hpp:106-121`), then the
    /// `bridge_distance` 16 mm segmentation pass (`hpp:189-215`); both
    /// passes filter insertions closer than `0.25 · flow_width` to either
    /// segment endpoint.
    fn annotate(&self, points: &[(f64, f64)], width: f32) -> Vec<AnnotatedPoint> {
        const EPSILON: f64 = 1e-4;
        const MAX_LINE_LENGTH: f64 = 16.0;
        let min_spacing = f64::from(width) * 0.25;
        let mut annotated = Vec::with_capacity(points.len() * 2);
        annotated.push(AnnotatedPoint {
            position: points[0],
            distance: self.nearest_previous(points[0]).0,
        });
        for &point in &points[1..] {
            let distance = self.nearest_previous(point).0;
            let back = *annotated.last().unwrap();
            if (f64::from(back.distance) > EPSILON) != (f64::from(distance) > EPSILON) {
                annotated.extend(
                    self.boundary_crossings(back.position, point)
                        .into_iter()
                        .filter(|position| between(back.position, *position, point, min_spacing))
                        .map(|position| AnnotatedPoint {
                            position,
                            distance: 0.0,
                        }),
                );
            }
            annotated.push(AnnotatedPoint {
                position: point,
                distance,
            });
        }
        let mut segmented = Vec::with_capacity(annotated.len() * 2);
        for index in 0..annotated.len() - 1 {
            let (current, next) = (annotated[index], annotated[index + 1]);
            segmented.push(current);
            let length_squared = {
                let (dx, dy) = (
                    next.position.0 - current.position.0,
                    next.position.1 - current.position.1,
                );
                dx * dx + dy * dy
            };
            let t = (MAX_LINE_LENGTH * MAX_LINE_LENGTH / length_squared).sqrt();
            let new_point_count = (1.0 / t) as usize;
            segmented.extend(
                (1..=new_point_count)
                    .map(|j| interpolate(current.position, next.position, f64::from(j as u32) * t))
                    .filter(|position| {
                        between(current.position, *position, next.position, min_spacing)
                    })
                    .map(|position| AnnotatedPoint {
                        position,
                        distance: self.nearest_previous(position).0,
                    }),
            );
        }
        segmented.push(*annotated.last().unwrap());
        segmented
    }

    /// `intersections_with_line<true>` crossings of the `back → point`
    /// segment against the previous layer's lines, widened to double
    /// (`hpp:110-116`).
    fn boundary_crossings(&self, back: (f64, f64), point: (f64, f64)) -> Vec<(f64, f64)> {
        let line = [
            [back.0 as f32, back.1 as f32],
            [point.0 as f32, point.1 as f32],
        ];
        self.previous_tree
            .intersections_sorted(line)
            .into_iter()
            .map(|(position, _)| (f64::from(position[0]), f64::from(position[1])))
            .collect()
    }

    /// Unsigned distance to the nearest previous-layer external line and
    /// that line's chained curl height
    /// (`prev_layer_lines.distance_from_lines_extra<false>`).
    fn nearest_previous(&self, point: (f64, f64)) -> (f32, f32) {
        // `AABBTreeLines.hpp:329-345`, `AABBTreeIndirect.hpp:552-632`:
        // strict improvement retains the first TREE-visited primitive.
        self.previous_tree
            .nearest_within([point.0 as f32, point.1 as f32], f32::INFINITY)
            .map_or((f32::INFINITY, 0.0), |hit| {
                (
                    hit.squared_distance.sqrt(),
                    self.previous[hit.index].curled_height,
                )
            })
    }
}

fn boundary_sign(
    tree: &LineDistanceTree<'_>,
    scale: CoordinateScale,
    middle: (f64, f64),
    width: f32,
) -> f64 {
    let signed = signed_distance(tree, scale, middle);
    if signed + f64::from(0.5_f32 * width) < 0.0 {
        -1.0
    } else {
        1.0
    }
}

/// `(a - b).norm() > min_spacing` — Eigen's `sqrt(squaredNorm)`, not
/// `hypot`.
fn spaced(a: (f64, f64), b: (f64, f64), min_spacing: f64) -> bool {
    let (dx, dy) = (a.0 - b.0, a.1 - b.1);
    (dx * dx + dy * dy).sqrt() > min_spacing
}

/// Both flanking distances of the inserted `mid` point exceed the
/// minimum spacing (`hpp:118-120, 209-211`).
fn between(start: (f64, f64), mid: (f64, f64), end: (f64, f64), min_spacing: f64) -> bool {
    spaced(mid, start, min_spacing) && spaced(end, mid, min_spacing)
}

/// `curr · (1 − t) + next · t` at double precision (`hpp:198`).
fn interpolate(current: (f64, f64), next: (f64, f64), t: f64) -> (f64, f64) {
    (
        current.0 * (1.0 - t) + next.0 * t,
        current.1 * (1.0 - t) + next.1 * t,
    )
}
