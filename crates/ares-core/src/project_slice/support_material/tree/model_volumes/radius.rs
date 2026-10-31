//! `TreeModelVolumes::ceilRadius` and `getRadiusNextCeil`.

use super::super::organic::TreeSettings;
use crate::geometry::{Coord, CoordinateScale};

pub(super) struct RadiusQuantization {
    radius_zero: Coord,
    xy_distance_delta: Coord,
    threshold: Coord,
    resolution: Coord,
    ignored_radii: Vec<Coord>,
}

impl RadiusQuantization {
    pub(super) fn prepare_tip_radii(&mut self, settings: &TreeSettings) {
        let mut possible_tip_radii = Vec::with_capacity((settings.tip_layers + 1) * 2);
        for distance in 0..=settings.tip_layers {
            let radius = settings.radius(distance, 0.0);
            possible_tip_radii.push(self.ceil_bucket(radius));
            possible_tip_radii.push(self.ceil_bucket(radius + self.xy_distance_delta));
        }
        possible_tip_radii.sort_unstable();
        possible_tip_radii.dedup();
        let mut radius = self.radius_zero;
        while radius <= settings.branch_radius {
            if possible_tip_radii.binary_search(&radius).is_err() {
                self.ignored_radii.push(radius);
            }
            radius = self.ceil_bucket(radius + 1);
        }
    }

    pub(super) fn new(
        scale: CoordinateScale,
        radius_zero: Coord,
        xy_distance_delta: Coord,
    ) -> Self {
        Self {
            radius_zero,
            xy_distance_delta,
            threshold: scale.scaled_delta(1.5) as Coord,
            resolution: scale.scaled_delta(0.5) as Coord,
            ignored_radii: Vec::new(),
        }
    }

    pub(super) fn ceil(&self, radius: Coord, min_xy_distance: bool) -> Coord {
        debug_assert!(radius >= 0);
        if min_xy_distance {
            self.ceil_bucket(radius)
        } else if radius > 0 {
            self.ceil_bucket(radius + self.xy_distance_delta)
        } else {
            self.xy_distance_delta
        }
    }

    pub(super) fn next_ceil(&self, radius: Coord, min_xy_distance: bool) -> Coord {
        debug_assert!(radius > 0);
        if min_xy_distance {
            self.ceil_bucket(radius)
        } else {
            self.ceil_bucket(radius + self.xy_distance_delta) - self.xy_distance_delta
        }
    }

    fn ceil_bucket(&self, radius: Coord) -> Coord {
        if radius == 0 {
            return 0;
        }
        if radius <= self.radius_zero {
            return self.radius_zero;
        }
        let mut output = self.radius_zero;
        let initial_delta = self.threshold - self.radius_zero;
        let ignored = |radius| self.ignored_radii.binary_search(&radius).is_ok();
        let (steps, step_size) = if initial_delta > self.resolution {
            let steps = (initial_delta + self.threshold - 1) / self.threshold;
            (steps, initial_delta / steps)
        } else {
            (0, self.resolution)
        };
        output += step_size;
        for _ in 0..steps {
            if output >= radius && !ignored(output) {
                return output;
            }
            output += step_size;
        }
        while output < radius || ignored(output) {
            debug_assert!(output as f64 * 1.5 > (output + self.resolution) as f64);
            // Upstream multiplies in double before truncating back to coord_t.
            output = (output as f64 * 1.5) as Coord;
        }
        output
    }
}
