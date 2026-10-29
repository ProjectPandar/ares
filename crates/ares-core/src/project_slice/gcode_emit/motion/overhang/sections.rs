//! Overhang speed sections — the reference-speed ladder and the
//! per-distance lookup (`speed_sections` family).

use super::super::options::MotionOptions;
use super::{BoundaryContext, ExtendedPoint, INTERSECTION_EPSILON_MM, OVERLAPS};
use crate::FloatOrPercent;

pub(super) fn speed_sections(
    width: f32,
    reference_speed: f64,
    options: &MotionOptions,
) -> Vec<(f32, f32)> {
    let reference_speed_f32 = reference_speed as f32;
    let band_speeds = options.overhang_speed_bands.map(|value| {
        value
            .map(|value| absolute(value, reference_speed))
            .filter(|speed| *speed >= 0.5)
            .map(|speed| source_dynamic_speed(speed, reference_speed, reference_speed_f32))
            .unwrap_or(reference_speed_f32)
    });
    let severe_speed = if options.slowdown_for_curled_perimeters {
        band_speeds[3]
    } else {
        source_dynamic_speed(options.bridge_speed, reference_speed, reference_speed_f32)
    };
    let speeds = [
        reference_speed_f32,
        band_speeds[0],
        band_speeds[1],
        band_speeds[2],
        band_speeds[3],
        severe_speed,
    ];
    let mut sections = OVERLAPS
        .into_iter()
        .zip(speeds)
        .map(|(overlap, speed)| ((f64::from(width) * (1.0 - overlap / 100.0)) as f32, speed))
        .collect::<Vec<_>>();
    sections.sort_by(|left, right| {
        left.0
            .total_cmp(&right.0)
            .then_with(|| right.1.total_cmp(&left.1))
    });
    for index in 1..sections.len() {
        if sections[index].0 == sections[index - 1].0 {
            sections[index].1 = sections[index - 1].1;
        }
    }
    sections
}

impl BoundaryContext<'_> {
    pub(super) fn add_boundary_intersections(
        &self,
        points: &[(f64, f64)],
        contour: Option<&[crate::project_slice::perimeters::classic::materialize::Point3]>,
    ) -> Vec<ExtendedPoint> {
        let mut result = Vec::with_capacity(points.len() * 2);
        result.push(self.extended(
            points[0],
            contour.map_or(0.0, |p| self.scale.unscale(p[0].z)),
        ));
        for (index, &point) in points.iter().enumerate().skip(1) {
            let next = self.extended(
                point,
                contour.map_or(0.0, |p| self.scale.unscale(p[index].z)),
            );
            let previous = *result.last().unwrap();
            self.append_intersections(&mut result, previous, next);
            result.push(next);
        }
        result
    }

    fn append_intersections(
        &self,
        result: &mut Vec<ExtendedPoint>,
        previous: ExtendedPoint,
        next: ExtendedPoint,
    ) {
        let threshold = f64::from(self.offset) + INTERSECTION_EPSILON_MM;
        let previous_outside = f64::from(previous.distance) > threshold;
        let next_outside = f64::from(next.distance) > threshold;
        if previous_outside == next_outside {
            return;
        }
        // AABBTreeLines.hpp:144-150: previous boundary boxes lie at Z=0.
        if previous.z.min(next.z) > 0.0 || previous.z.max(next.z) < 0.0 {
            return;
        }
        let line = [[previous.x, previous.y], [next.x, next.y]];
        for (intersection, _) in self.tree.intersections_sorted_mm(line, self.scale.factor()) {
            // Line.hpp:123-145 uses the XY intersection parameter for all XYZ axes.
            let delta = [next.x - previous.x, next.y - previous.y];
            let axis = usize::from(delta[1].abs() > delta[0].abs());
            let ratio = (intersection[axis] - line[0][axis]) / delta[axis];
            let intersection = ExtendedPoint {
                x: intersection[0],
                y: intersection[1],
                z: previous.z + ratio * (next.z - previous.z),
                distance: self.offset,
            };
            if distance(previous, intersection) > self.minimum_spacing
                && distance(intersection, next) > self.minimum_spacing
            {
                result.push(intersection);
            }
        }
    }

    pub(super) fn add_segmentation_points(
        &self,
        points: &[ExtendedPoint],
        minimum_slowdown_distance: f32,
    ) -> Vec<ExtendedPoint> {
        let mut result = Vec::with_capacity(points.len() * 2);
        result.push(points[0]);
        for pair in points.windows(2) {
            let current = pair[0];
            let next = pair[1];
            let line_length = distance(current, next);
            let near_boundary = |point: ExtendedPoint| {
                point.distance > -self.offset && point.distance < self.offset + 2.0
            };
            let needs_slowdown = current.distance.abs() > minimum_slowdown_distance
                || next.distance.abs() > minimum_slowdown_distance;
            let should_segment = (near_boundary(current) || near_boundary(next))
                && ((minimum_slowdown_distance > 0.0 && needs_slowdown && line_length >= 2.0)
                    || (minimum_slowdown_distance <= 0.0 && line_length > 4.0));
            if should_segment {
                let a0 = (f64::from(current.distance + 3.0_f32 * self.offset) / line_length)
                    .clamp(0.0, 1.0);
                let a1 = (1.0 - f64::from(next.distance + 3.0_f32 * self.offset) / line_length)
                    .clamp(0.0, 1.0);
                self.append_segmentation_candidate(
                    &mut result,
                    [current, next],
                    a0.min(a1),
                    minimum_slowdown_distance,
                );
                self.append_segmentation_candidate(
                    &mut result,
                    [current, next],
                    a0.max(a1),
                    minimum_slowdown_distance,
                );
            }
            result.push(next);
        }
        result
    }

    fn append_segmentation_candidate(
        &self,
        result: &mut Vec<ExtendedPoint>,
        endpoints: [ExtendedPoint; 2],
        factor: f64,
        minimum_slowdown_distance: f32,
    ) {
        let [current, next] = endpoints;
        if factor <= 0.0 || factor >= 1.0 {
            return;
        }
        let candidate = interpolate(current, next, factor);
        let raw_distance = self.signed_distance((candidate.x, candidate.y), candidate.z);
        let candidate = ExtendedPoint {
            distance: (raw_distance + f64::from(self.offset)) as f32,
            ..candidate
        };
        if raw_distance.abs() > f64::from(minimum_slowdown_distance)
            && distance(current, candidate) > self.minimum_spacing
            && distance(candidate, next) > self.minimum_spacing
        {
            result.push(candidate);
        }
    }

    fn extended(&self, point: (f64, f64), z: f64) -> ExtendedPoint {
        // `ExtendedPoint.distance` is `float`, but the boundary offset
        // add happens in the f64 distance domain before narrowing
        // (`next_point.distance = distance + boundary_offset`,
        // `ExtrusionProcessor.hpp:100-101`).
        let raw_distance = self.signed_distance(point, z);
        ExtendedPoint {
            x: point.0,
            y: point.1,
            z,
            distance: (raw_distance + f64::from(self.offset)) as f32,
        }
    }

    fn signed_distance(&self, point: (f64, f64), z: f64) -> f64 {
        // `distance_from_lines_extra<true>` over `LinesDistancer<Linef3>`
        // (`ExtrusionProcessor.hpp:92-101,366`): the previous-layer
        // boundary lines are unscaled f64 — the distance is measured in
        // f64 millimetres, then signed by the inside/outside winding
        // (negative inside).
        let nearest = self
            .tree
            .nearest_mm([point.0, point.1], self.scale.factor())
            .expect("a nonempty boundary has a nearest line");
        self.tree
            .outside_mm([point.0, point.1], self.scale.factor()) as f64
            * if z == 0.0 {
                nearest.squared_distance.sqrt()
            } else {
                (nearest.squared_distance + z * z).sqrt()
            }
    }
}

pub(super) fn speed_for_distance(
    distance: f32,
    sections: &[(f32, f32)],
    original_speed: f32,
) -> f32 {
    if distance <= sections[0].0 {
        return original_speed.round();
    }
    if distance >= sections[sections.len() - 1].0 {
        return sections[sections.len() - 1].1.round();
    }
    let upper = sections.partition_point(|section| distance > section.0);
    let lower = upper - 1;
    let ratio = (distance - sections[lower].0) / (sections[upper].0 - sections[lower].0);
    ((1.0_f32 - ratio) * sections[lower].1 + ratio * sections[upper].1).round()
}

pub(super) fn source_dynamic_speed(
    speed: f64,
    reference_speed: f64,
    reference_speed_f32: f32,
) -> f32 {
    (f64::from(reference_speed_f32) * (speed * 100.0 / reference_speed) / 100.0) as f32
}

pub(super) fn absolute(value: FloatOrPercent, base: f64) -> f64 {
    match value {
        FloatOrPercent::Float(value) => value,
        FloatOrPercent::Percent(value) => base * value.0 / 100.0,
    }
}

pub(super) fn scale_trunc(value: f64, scale: crate::geometry::CoordinateScale) -> i64 {
    (value / scale.factor()) as i64
}

pub(super) fn interpolate(
    first: ExtendedPoint,
    second: ExtendedPoint,
    factor: f64,
) -> ExtendedPoint {
    ExtendedPoint {
        x: first.x + factor * (second.x - first.x),
        y: first.y + factor * (second.y - first.y),
        z: first.z + factor * (second.z - first.z),
        distance: 0.0,
    }
}

pub(super) fn distance(first: ExtendedPoint, second: ExtendedPoint) -> f64 {
    let xy = (second.x - first.x).hypot(second.y - first.y);
    if first.z == second.z {
        xy
    } else {
        let dx = second.x - first.x;
        let dy = second.y - first.y;
        let dz = second.z - first.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}
