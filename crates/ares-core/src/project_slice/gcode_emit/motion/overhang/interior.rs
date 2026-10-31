//! ExtrusionProcessor.hpp:174–261: midpoint discovery and in-place sample retention.

use super::printing_state::PrintingState;
use super::sections::{distance, interpolate};
use super::{BoundaryContext, ExtendedPoint};

struct Subspan {
    start: f64,
    end: f64,
    depth: i32,
}

impl BoundaryContext<'_> {
    pub(super) fn add_interior_points(
        &self,
        points: Vec<ExtendedPoint>,
        minimum_slowdown_distance: f32,
        printing: &PrintingState<'_>,
    ) -> Vec<ExtendedPoint> {
        if minimum_slowdown_distance <= 0.0 && printing.fan_threshold < 0.0 {
            return points;
        }
        let maximum_spacing = 2.0_f64.max(4.0 * self.minimum_spacing);
        let mut sampled_points = Vec::new();
        let mut interior = Vec::new();
        let mut pending = Vec::new();
        for (index, pair) in points.windows(2).enumerate() {
            let endpoints = [pair[0], pair[1]];
            self.discover_samples(endpoints, maximum_spacing, &mut interior, &mut pending);
            retain_samples(&mut interior, endpoints, printing);
            if !interior.is_empty() && sampled_points.is_empty() {
                sampled_points.reserve(points.len() + 8);
                sampled_points.extend_from_slice(&points[..=index]);
            }
            if !sampled_points.is_empty() {
                sampled_points.extend(interior.iter().map(|&(factor, distance)| ExtendedPoint {
                    distance,
                    ..interpolate(endpoints[0], endpoints[1], factor)
                }));
                sampled_points.push(endpoints[1]);
            }
        }
        if sampled_points.is_empty() {
            points
        } else {
            sampled_points
        }
    }

    fn discover_samples(
        &self,
        endpoints: [ExtendedPoint; 2],
        maximum_spacing: f64,
        interior: &mut Vec<(f64, f32)>,
        pending: &mut Vec<Subspan>,
    ) {
        interior.clear();
        let length = distance(endpoints[0], endpoints[1]);
        if length >= maximum_spacing {
            pending.push(Subspan {
                start: 0.0,
                end: 1.0,
                depth: 10,
            });
        }
        while let Some(span) = pending.pop() {
            if span.depth <= 0 || (span.end - span.start) * length < maximum_spacing {
                continue;
            }
            let middle = 0.5 * (span.start + span.end);
            let point = self.sample(interpolate(endpoints[0], endpoints[1], middle));
            interior.push((middle, point.distance));
            pending.push(Subspan {
                start: span.start,
                end: middle,
                depth: span.depth - 1,
            });
            pending.push(Subspan {
                start: middle,
                end: span.end,
                depth: span.depth - 1,
            });
        }
        interior.sort_unstable_by(|left, right| left.0.total_cmp(&right.0));
    }
}

fn retain_samples(
    interior: &mut Vec<(f64, f32)>,
    endpoints: [ExtendedPoint; 2],
    printing: &PrintingState<'_>,
) {
    let mut kept = 0;
    for index in 0..interior.len() {
        let sample = interior[index].1;
        let at_start = kept == 0;
        let at_end = index + 1 == interior.len();
        let before = if at_start {
            endpoints[0].distance
        } else {
            interior[kept - 1].1
        };
        let after = if at_end {
            endpoints[1].distance
        } else {
            interior[index + 1].1
        };
        let worth_before = !printing.same(sample, before)
            && (!at_start || printing.slower_or_cooled(sample, before));
        let worth_after =
            !printing.same(sample, after) && (!at_end || printing.slower_or_cooled(sample, after));
        if worth_before || worth_after {
            interior[kept] = interior[index];
            kept += 1;
        }
    }
    interior.truncate(kept);
}
