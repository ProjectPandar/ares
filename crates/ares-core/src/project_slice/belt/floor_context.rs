//! `Support/BeltFloorContext.cpp`: the floor shared by belt brim and support.

use super::{BeltFloor, EPSILON};
use crate::geometry::{ClipperError, CoordinateScale, Point, Polygon, difference_polygons_paths};

#[derive(Clone, Copy)]
pub(in crate::project_slice) struct BeltFloorContext {
    floor: BeltFloor,
    offset: f64,
}

impl BeltFloorContext {
    pub(in crate::project_slice) fn new(floor: BeltFloor, offset: f64) -> Self {
        Self { floor, offset }
    }

    pub(in crate::project_slice) fn local(mut self, print_z_offset: f64) -> Self {
        self.floor.shift -= print_z_offset;
        self
    }

    pub(in crate::project_slice) fn propagates(self) -> bool {
        self.floor.shear.abs() > EPSILON
    }

    /// `TreeSupport.cpp:945–950`: advance a lower slice along signed belt shear.
    pub(in crate::project_slice) fn lower_layer_shift(
        self,
        height: f64,
        scale: CoordinateScale,
    ) -> Point {
        let shift = -scale.scaled_delta(height * self.floor.shear) as i64;
        if self.floor.axis == 0 {
            Point::new(shift, 0)
        } else {
            Point::new(0, shift)
        }
    }

    pub(super) fn cutoff_u(self, print_z: f64) -> f64 {
        (print_z - self.floor.shift - self.offset) / self.floor.shear
    }

    pub(super) fn floor_print_z(self, u: f64) -> f64 {
        self.floor.shear * u + self.offset + self.floor.shift
    }

    pub(in crate::project_slice) fn half_plane(
        self,
        print_z: f64,
        scale: CoordinateScale,
        belt_surface: bool,
    ) -> [Polygon; 1] {
        let cutoff = scale.scaled_delta(self.cutoff_u(print_z)) as i64;
        let bound = scale.scaled_delta(1000.0) as i64;
        let high_side = (self.floor.shear > 0.0) == belt_surface;
        let axis_bounds = if high_side {
            [cutoff, bound]
        } else {
            [-bound, cutoff]
        };
        let ([left, right], [bottom, top]) = if self.floor.axis == 0 {
            (axis_bounds, [-bound, bound])
        } else {
            ([-bound, bound], axis_bounds)
        };
        [Polygon::new(vec![
            Point::new(left, bottom),
            Point::new(right, bottom),
            Point::new(right, top),
            Point::new(left, top),
        ])]
    }

    pub(in crate::project_slice) fn trim<'a>(
        self,
        layers: impl IntoIterator<Item = (f64, &'a mut Vec<Polygon>)>,
        scale: CoordinateScale,
    ) -> Result<(), ClipperError> {
        for (print_z, polygons) in layers {
            *polygons =
                difference_polygons_paths(polygons, &self.half_plane(print_z, scale, true))?;
        }
        Ok(())
    }
}
