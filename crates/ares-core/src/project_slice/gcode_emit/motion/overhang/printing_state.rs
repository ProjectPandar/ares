//! ExtrusionProcessor.hpp:154–172,600–630: captured effective-speed/fan predicates.

use super::sections::speed_for_distance;

pub(super) struct PrintingState<'a> {
    pub(super) sections: &'a [(f32, f32)],
    pub(super) original_speed: f32,
    pub(super) width_inverse: f32,
    pub(super) fan_threshold: f32,
}

impl PrintingState<'_> {
    fn effective_speed(&self, distance: f32) -> f32 {
        speed_for_distance(distance, self.sections, self.original_speed).min(self.original_speed)
    }

    fn fan_on(&self, distance: f32) -> bool {
        self.fan_threshold >= 0.0 && 1.0_f32 - distance * self.width_inverse <= self.fan_threshold
    }

    pub(super) fn same(&self, first: f32, second: f32) -> bool {
        (self.effective_speed(first) - self.effective_speed(second)).abs() <= 1.0
            && self.fan_on(first) == self.fan_on(second)
    }

    pub(super) fn slower_or_cooled(&self, first: f32, second: f32) -> bool {
        self.effective_speed(first) < self.effective_speed(second) - 1.0
            || (self.fan_on(first) && !self.fan_on(second))
    }
}
