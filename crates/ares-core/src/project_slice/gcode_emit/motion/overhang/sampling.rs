//! ExtrusionProcessor.hpp:263–328: printing-state-aware two-cut segmentation.

use super::printing_state::PrintingState;
use super::sections::{distance, interpolate};
use super::{BoundaryContext, ExtendedPoint};

impl BoundaryContext<'_> {
    pub(super) fn sample(&self, point: ExtendedPoint) -> ExtendedPoint {
        ExtendedPoint {
            distance: (self.signed_distance((point.x, point.y), point.z) + f64::from(self.offset))
                as f32,
            ..point
        }
    }

    pub(super) fn add_segmentation_points(
        &self,
        points: &[ExtendedPoint],
        printing: &PrintingState<'_>,
    ) -> Vec<ExtendedPoint> {
        let mut result = Vec::with_capacity(points.len() * 2);
        result.push(points[0]);
        for pair in points.windows(2) {
            let [current, next] = [pair[0], pair[1]];
            let near_boundary = |point: ExtendedPoint| {
                point.distance > -self.offset && point.distance < self.offset + 2.0_f32
            };
            let length = distance(current, next);
            if (near_boundary(current) || near_boundary(next))
                && length >= 2.0
                && (printing.slower_or_cooled(current.distance, 0.0)
                    || printing.slower_or_cooled(next.distance, 0.0))
            {
                let (cuts, keep) = self.segment_cuts([current, next], length, printing);
                result.extend(
                    cuts.into_iter()
                        .zip(keep)
                        .filter_map(|(cut, keep)| keep.then_some(cut)),
                );
            }
            result.push(next);
        }
        result
    }

    fn segment_cuts(
        &self,
        [current, next]: [ExtendedPoint; 2],
        length: f64,
        printing: &PrintingState<'_>,
    ) -> ([ExtendedPoint; 2], [bool; 2]) {
        let first = (f64::from(current.distance + 3.0_f32 * self.offset) / length).clamp(0.0, 1.0);
        let second =
            (1.0 - f64::from(next.distance + 3.0_f32 * self.offset) / length).clamp(0.0, 1.0);
        let factors = [first.min(second), first.max(second)];
        let mut cuts = [current; 2];
        let mut keep = [false; 2];
        for index in 0..2 {
            let factor = factors[index];
            if if index == 0 {
                factor >= 1.0
            } else {
                factor <= 0.0
            } {
                continue;
            }
            cuts[index] = self.sample(interpolate(current, next, factor));
            keep[index] = distance(cuts[index], current) > self.minimum_spacing
                && distance(next, cuts[index]) > self.minimum_spacing;
        }
        // The second decision observes the updated first decision, as upstream does.
        if keep[0] {
            keep[0] = !printing.same(cuts[0].distance, current.distance)
                || !printing.same(
                    cuts[0].distance,
                    if keep[1] {
                        cuts[1].distance
                    } else {
                        next.distance
                    },
                );
        }
        if keep[1] {
            keep[1] = !printing.same(
                cuts[1].distance,
                if keep[0] {
                    cuts[0].distance
                } else {
                    current.distance
                },
            ) || !printing.same(cuts[1].distance, next.distance);
        }
        if keep[0] && keep[1] && distance(cuts[1], cuts[0]) <= self.minimum_spacing {
            keep[1] = false;
        }
        (cuts, keep)
    }
}
