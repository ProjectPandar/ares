//! The `PartPlateList` steps around an arrange (`PartPlate.cpp` at v2.5.0-beta `345efec64`):
//! the exclusion areas as fixed items, the bed index of an arranged item, its move onto that
//! plate, and the plate list rebuilt from the arranged instances.

use std::collections::BTreeSet;

use super::{Plate, PlateList, PlateModel, origin};
use crate::arrangement::{ArrangePolygon, SCALING_FACTOR, UNARRANGED};
use crate::geometry::{ExPolygon, Point, Polygon};
use crate::project::compute_colum_count;

/// `PartPlateList::MAX_PLATES_COUNT` (`PartPlate.hpp`).
const MAX_PLATES_COUNT: i32 = 36;
/// `LOGICAL_PART_PLATE_GAP` (`PartPlate.hpp`).
const LOGICAL_PART_PLATE_GAP: f64 = 1.0 / 5.0;

/// `Point(double, double)`: rounded.
fn rounded(x: f64, y: f64) -> Point {
    Point::new(
        (x / SCALING_FACTOR).round() as i64,
        (y / SCALING_FACTOR).round() as i64,
    )
}

impl PlateList {
    fn stride(&self) -> (f64, f64) {
        (
            f64::from(self.size.0) * (1.0 + LOGICAL_PART_PLATE_GAP),
            f64::from(self.size.1) * (1.0 + LOGICAL_PART_PLATE_GAP),
        )
    }

    /// `preprocess_exclude_areas` (`PartPlate.cpp:7692-7790`) without exclusion volumes: the
    /// clumping-detection area and each four-point `bed_exclude_area` box of the first plate,
    /// as fixed virtual items on `num_plates` beds.
    pub(in crate::plater) fn exclude_areas(
        wrapping_area: Option<&[[f64; 2]]>,
        bed_exclude_area: &[[f64; 2]],
        num_plates: i32,
        inflation: i64,
    ) -> Vec<ArrangePolygon> {
        let mut regions = Vec::new();
        if let Some(area) = wrapping_area.filter(|area| !area.is_empty()) {
            let contour = Polygon::new(area.iter().map(|p| rounded(p[0], p[1])).collect());
            regions.push((contour, "WrappingRegion".to_owned()));
        }
        // `PartPlate::calc_bounding_boxes` (`:626-643`): every four points make a box.
        for (index, corners) in bed_exclude_area.as_chunks::<4>().0.iter().enumerate() {
            let (mut min, mut max) = ([f64::MAX; 2], [f64::MIN; 2]);
            for p in corners {
                min = [min[0].min(p[0]), min[1].min(p[1])];
                max = [max[0].max(p[0]), max[1].max(p[1])];
            }
            let contour = Polygon::new(vec![
                rounded(min[0], min[1]),
                rounded(max[0], min[1]),
                rounded(max[0], max[1]),
                rounded(min[0], max[1]),
            ]);
            regions.push((contour, format!("ExcludedRegion{index}")));
        }
        let mut items = Vec::new();
        for (contour, name) in regions {
            for bed in 0..num_plates {
                items.push(ArrangePolygon {
                    poly: ExPolygon::new(contour.clone(), Vec::new()),
                    is_virt_object: true,
                    bed_idx: bed,
                    height: 1.0,
                    name: name.clone(),
                    inflation,
                    ..ArrangePolygon::default()
                });
            }
        }
        items
    }

    /// `clear(false, false, true, -1)` (`PartPlate.cpp:6454-6501`): no plate holds an instance.
    pub(in crate::plater) fn clear_instances(&mut self) {
        for plate in &mut self.plates {
            plate.instances.clear();
        }
        self.unprintable.instances.clear();
    }

    /// `postprocess_bed_index_for_selected` (`PartPlate.cpp:7833-7880`) without locked plates:
    /// plates are created until the item's bed exists.
    pub(in crate::plater) fn create_plates_for(&mut self, ap: &ArrangePolygon) {
        if ap.bed_idx == -1 || (ap.bed_idx as usize) < self.plates.len() {
            return;
        }
        while let Some(index) = self.create_unadjusted_plate() {
            if ap.bed_idx as usize <= index {
                break;
            }
        }
    }

    /// `create_plate(false)` (`PartPlate.cpp:6585-6669`): a reflowed grid moves the plates but
    /// not their instances.
    fn create_unadjusted_plate(&mut self) -> Option<usize> {
        let new_index = self.plates.len();
        if new_index >= MAX_PLATES_COUNT as usize {
            return None;
        }
        let cols = compute_colum_count(new_index + 1);
        let old_cols = compute_colum_count(new_index);
        self.plates.push(Plate {
            origin: origin(new_index, cols, self.size),
            instances: BTreeSet::new(),
            source: None,
        });
        self.cols = compute_colum_count(self.plates.len());
        if old_cols != cols {
            for index in 0..self.plates.len() {
                self.plates[index].origin = origin(index, self.cols, self.size);
            }
        }
        self.unprintable.origin = self.origin_for_unprintable();
        Some(new_index)
    }

    /// `postprocess_arrange_polygon` (`PartPlate.cpp:7931-7952`): the item moved onto its
    /// bed's plate in the grid.
    pub(in crate::plater) fn move_to_plate(&self, ap: &mut ArrangePolygon, selected: bool) {
        if ap.bed_idx == UNARRANGED || !(selected || ap.bed_idx != MAX_PLATES_COUNT) {
            return;
        }
        let (row, col) = (ap.bed_idx / self.cols as i32, ap.bed_idx % self.cols as i32);
        ap.row = row;
        ap.col = col;
        let (stride_x, stride_y) = self.stride();
        // `translation += scaled<double>(...)`: the sum truncates to `coord_t`.
        let x = (ap.translation.x() as f64 + stride_x * f64::from(col) / SCALING_FACTOR) as i64;
        let y = (ap.translation.y() as f64 - stride_y * f64::from(row) / SCALING_FACTOR) as i64;
        ap.translation = Point::new(x, y);
    }

    /// `reload_all_objects(true, -1)` (`PartPlate.cpp:7498-7546`): each instance on the first
    /// plate it reaches, else on the unprintable plate.
    pub(in crate::plater) fn reload_all_objects(&mut self, model: &impl PlateModel) {
        self.clear_instances();
        for key in model.keys() {
            let bbox = model.convex_hull_bounding_box(key);
            match (0..self.plates.len())
                .find(|&index| self.intersects(&self.plates[index], true, &bbox))
            {
                Some(index) => {
                    self.plates[index].instances.insert(key);
                }
                None if self.intersects(&self.unprintable, false, &bbox) => {
                    self.unprintable.instances.insert(key);
                }
                None => {}
            }
        }
    }
}
