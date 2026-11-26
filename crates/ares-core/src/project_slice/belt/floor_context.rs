//! `Support/BeltFloorContext.cpp`: the print-space floor used by belt brim.

use super::BeltFloor;

#[derive(Clone, Copy)]
pub(super) struct BeltFloorContext {
    floor: BeltFloor,
    offset: f64,
}

impl BeltFloorContext {
    pub(super) fn new(floor: BeltFloor, offset: f64) -> Self {
        Self { floor, offset }
    }

    pub(super) fn cutoff_u(self, print_z: f64) -> f64 {
        (print_z - self.floor.shift - self.offset) / self.floor.shear
    }

    pub(super) fn floor_print_z(self, u: f64) -> f64 {
        self.floor.shear * u + self.offset + self.floor.shift
    }
}
