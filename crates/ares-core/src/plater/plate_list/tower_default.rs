//! The tower position of a new plate (`PartPlateList::set_default_wipe_tower_pos_for_plate`,
//! `PartPlate.cpp:6339-6428` at v2.5.0-beta `345efec64`) with
//! `WipeTower::move_box_inside_polygon` (`GCode/WipeTower.cpp:1826-1874`).

use super::Towers;
use super::wipe_tower::{WIPE_TOWER_AUTO_MARGIN, scaled, scaled_f32};
use crate::SliceError;
use crate::geometry::{ExPolygon, Point, Polygon, difference_ex_polygons};

/// `PartPlate.cpp:142-148`.
const WIPE_TOWER_DEFAULT_POS: (f32, f32) = (165.0, 250.0);
const I3_WIPE_TOWER_DEFAULT_POS: (f32, f32) = (0.0, 250.0);
/// `EPSILON` (`libslic3r.h`).
const EPSILON: f64 = 1e-4;

impl Towers<'_> {
    /// `set_default_wipe_tower_pos_for_plate(index, true)`: the new plate's tower position.
    pub(in crate::plater) fn default_position(&self, index: usize) -> Result<[f64; 2], SliceError> {
        let structure = self.config.option("printer_structure");
        // `get_printer_type` finds no vendor model in a session, so the N9 row never applies.
        let (mut x, mut y) = if structure.key() == "i3" {
            I3_WIPE_TOWER_DEFAULT_POS
        } else {
            WIPE_TOWER_DEFAULT_POS
        };
        let origin = self.plates.plates[index].origin;
        let (shape_min, shape_max) = self.plates.shape;
        // `get_bounding_box` in the plate's frame; the session has no per-nozzle areas.
        let (x_min, x_max, y_max) = (shape_min[0], shape_max[0], shape_max[1]);
        let footprint = self.footprint(index, 2, false)?;
        let margin = (WIPE_TOWER_AUTO_MARGIN + footprint.brim_width as f32 as f64) as f32;
        if f64::from(x + margin) + footprint.width > x_max {
            x = (x_max - footprint.width - f64::from(margin)) as f32;
        } else if f64::from(x) < f64::from(margin) + x_min {
            x = (f64::from(margin) + x_min) as f32;
        }
        if f64::from(y + margin) + footprint.depth > y_max {
            y = (y_max - footprint.depth - f64::from(margin)) as f32;
        } else if y < margin {
            y = margin;
        }
        // `get_shared_printable_polygon` (`Polygon::new_scale(m_shape)`, the shape at the plate's
        // origin) moved into the plate's frame.
        let (dx, dy) = (scaled(origin[0]), scaled(origin[1]));
        let bed = Polygon::new(
            self.project
                .settings()
                .printer
                .remaining
                .printable_area
                .0
                .iter()
                .map(|p| Point::new(scaled(p.x + origin[0]) - dx, scaled(p.y + origin[1]) - dy))
                .collect(),
        );
        let tower = (
            Point::new(scaled(f64::from(x)), scaled(f64::from(y))),
            Point::new(
                scaled(f64::from(x) + footprint.width),
                scaled(f64::from(y) + footprint.depth),
            ),
        );
        let shift = move_box_inside_polygon(tower, &bed, scaled_f32(margin));
        Ok([f64::from(x + shift[0]), f64::from(y + shift[1])])
    }
}

/// `WipeTower::move_box_inside_polygon` for one polygon: the shift that brings `(min, max)` at
/// least `offset` inside `bed`.
fn move_box_inside_polygon((min, max): (Point, Point), bed: &Polygon, offset: i64) -> [f32; 2] {
    let points = bed.points();
    let bed_min = [
        points.iter().map(|p| p.x()).min().expect("bed"),
        points.iter().map(|p| p.y()).min().expect("bed"),
    ];
    let bed_max = [
        points.iter().map(|p| p.x()).max().expect("bed"),
        points.iter().map(|p| p.y()).max().expect("bed"),
    ];
    let (box_min, box_max) = ([min.x(), min.y()], [max.x(), max.y()]);
    if box_max[0] - box_min[0] >= bed_max[0] - bed_min[0] - 2 * offset
        || box_max[1] - box_min[1] >= bed_max[1] - bed_min[1] - 2 * offset
    {
        return [0.0, 0.0];
    }
    let mut shift = [0_i64; 2];
    for axis in 0..2 {
        if box_max[axis] > bed_max[axis] - offset {
            shift[axis] = (bed_max[axis] - offset) - box_max[axis];
        } else if box_min[axis] < bed_min[axis] + offset {
            shift[axis] = (bed_min[axis] + offset) - box_min[axis];
        }
    }
    // `unscaled<float>`: multiplied in float.
    let unscaled_shift = |shift: [i64; 2]| [shift[0] as f32 * 1e-6_f32, shift[1] as f32 * 1e-6_f32];
    let area = bed.area().abs();
    let bed_area = (bed_max[0] - bed_min[0]) as f64 * (bed_max[1] - bed_min[1]) as f64;
    if area >= bed_area * (1.0 - EPSILON) {
        return unscaled_shift(shift);
    }
    // `BoundingBox::inflated(max(offset, 0) - SCALED_EPSILON)`, moved by the shift.
    let grow = offset.max(0) - scaled(EPSILON);
    let padded = (
        [box_min[0] - grow + shift[0], box_min[1] - grow + shift[1]],
        [box_max[0] + grow + shift[0], box_max[1] + grow + shift[1]],
    );
    let fits = |by: [i64; 2]| {
        let (lo, hi) = (
            Point::new(padded.0[0] + by[0], padded.0[1] + by[1]),
            Point::new(padded.1[0] + by[0], padded.1[1] + by[1]),
        );
        let moved = Polygon::new(vec![
            lo,
            Point::new(hi.x(), lo.y()),
            hi,
            Point::new(lo.x(), hi.y()),
        ]);
        difference_ex_polygons(
            &[ExPolygon::new(moved, Vec::new())],
            std::slice::from_ref(bed),
        )
        .expect("tower fit")
        .is_empty()
    };
    if fits([0, 0]) {
        return unscaled_shift(shift);
    }
    let center = |lo: i64, hi: i64| ((i128::from(lo) + i128::from(hi)) / 2) as i64;
    let mut lo = [0_i64; 2];
    let mut hi = [
        center(bed_min[0], bed_max[0]) - center(padded.0[0], padded.1[0]),
        center(bed_min[1], bed_max[1]) - center(padded.0[1], padded.1[1]),
    ];
    if !fits(hi) {
        return unscaled_shift(shift);
    }
    for _ in 0..12 {
        let mid = [(lo[0] + hi[0]) / 2, (lo[1] + hi[1]) / 2];
        if fits(mid) {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    unscaled_shift([shift[0] + hi[0], shift[1] + hi[1]])
}
