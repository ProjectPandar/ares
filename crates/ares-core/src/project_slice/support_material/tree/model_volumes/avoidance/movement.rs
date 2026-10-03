//! Float movement arithmetic and the explicit `round_up_divide<int>` call.

use crate::geometry::Coord;

pub(super) struct Movement {
    pub(super) count: i32,
    pub(super) step: f32,
    pub(super) last_step: f32,
}

impl Movement {
    pub(super) fn offset(&self, index: i32) -> f32 {
        if index + 1 == self.count {
            -self.last_step
        } else {
            -self.step
        }
    }

    pub(super) fn new(radius: Coord, xy: Coord, maximum: Coord, minimum_last: f32) -> Option<Self> {
        let maximum = maximum as f32;
        let mut step = (1.9 * radius.max(xy) as f64) as f32;
        if f64::from(step) < 0.0001 {
            return None;
        }
        // Explicit C++ template arguments narrow both float operands before division.
        let divisor = step as i32;
        let mut count = (maximum as i32 + divisor - 1) / divisor;
        let mut last_step = maximum - (count - 1) as f32 * step;
        if last_step < minimum_last && count > 1 {
            count -= 1;
            step = maximum / count as f32;
            last_step = step;
        }
        Some(Self {
            count,
            step,
            last_step,
        })
    }
}
