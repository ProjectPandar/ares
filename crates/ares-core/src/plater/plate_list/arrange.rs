//! The `PartPlateList` steps around an arrange (`PartPlate.cpp` at v2.5.0-beta `345efec64`):
//! the exclusion areas as fixed items, the bed index of an arranged item, its move onto that
//! plate, and the plate list rebuilt from the arranged instances.

use std::collections::BTreeSet;

use super::{BoundingBox, InstanceKey, Plate, PlateList, PlateModel, origin};
use crate::arrangement::{
    ArrangePolygon, SCALING_FACTOR, UNARRANGED, bed_exclusion_arrange_polygon,
};
use crate::bed_exclusion::{
    active_bed_exclude_volume_mode, get_bed_excluded_regions_by_extruder, has_bed_exclude_volumes,
};
use crate::geometry::{ExPolygon, Point, Polygon};
use crate::project::compute_colum_count;
use crate::{BedExcludeVolumeMode, DynamicPrintConfig};

/// `PartPlateList::MAX_PLATES_COUNT` (`PartPlate.hpp`).
const MAX_PLATES_COUNT: i32 = 36;
/// `EPSILON` (`libslic3r.h`).
const EPSILON: f64 = 1e-4;
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
        config: &DynamicPrintConfig,
        num_plates: i32,
        inflation: i64,
    ) -> Vec<ArrangePolygon> {
        let per_plate = |contour: Polygon, name: String| {
            (0..num_plates).map(move |bed| ArrangePolygon {
                poly: ExPolygon::new(contour.clone(), Vec::new()),
                is_virt_object: true,
                bed_idx: bed,
                height: 1.0,
                name: name.clone(),
                inflation,
                ..ArrangePolygon::default()
            })
        };
        let mut items = Vec::new();
        if let Some(area) = wrapping_area.filter(|area| !area.is_empty()) {
            let contour = Polygon::new(area.iter().map(|p| rounded(p[0], p[1])).collect());
            items.extend(per_plate(contour, "WrappingRegion".to_owned()));
        }
        items.extend(bed_exclusion_blockers(config, num_plates, inflation));
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
            items.extend(per_plate(contour, format!("ExcludedRegion{index}")));
        }
        items
    }

    /// `contain_instance(key) || intersect_instance(key)` of the plate at `index`, with the
    /// instance's convex hull box `bbox` (`PartPlate.cpp:4459-4470, 4627-4649`): the test
    /// `prepare_partplate` selects by.
    pub(in crate::plater) fn holds_or_reaches(
        &self,
        index: usize,
        key: InstanceKey,
        bbox: &BoundingBox,
    ) -> bool {
        let plate = &self.plates[index];
        plate.instances.contains(&key) || self.intersects(plate, true, bbox)
    }

    /// `preprocess_arrange_polygon_other_locked` (`PartPlate.cpp:7663-7690`): an instance off
    /// the current plate is locked, moved to its plate's bed or, on no plate, given bed
    /// `MAX_PLATES_COUNT`.
    pub(in crate::plater) fn lock_off_current_plate(
        &self,
        key: InstanceKey,
        ap: &mut ArrangePolygon,
        selected: bool,
    ) -> bool {
        if selected {
            return false;
        }
        let Some(index) = self.holding_plate(key) else {
            ap.bed_idx = MAX_PLATES_COUNT;
            return true;
        };
        self.move_to_plate_bed(ap, index);
        true
    }

    /// `preprocess_arrange_polygon(..., true)` (`PartPlate.cpp:7617-7660`): an instance on a
    /// locked plate is locked, moved to that plate's bed.
    pub(in crate::plater) fn lock_on_locked_plate(
        &self,
        key: InstanceKey,
        ap: &mut ArrangePolygon,
    ) -> bool {
        match self.holding_plate(key) {
            Some(index) if self.plates[index].locked => {
                self.move_to_plate_bed(ap, index);
                true
            }
            _ => false,
        }
    }

    /// The first plate holding `key` (`contain_instance`).
    fn holding_plate(&self, key: InstanceKey) -> Option<usize> {
        self.plates
            .iter()
            .position(|plate| plate.instances.contains(&key))
    }

    /// The item on the bed of the plate at `index`, its translation made plate-local.
    fn move_to_plate_bed(&self, ap: &mut ArrangePolygon, index: usize) {
        let index = index as i32;
        let (row, col) = (index / self.cols as i32, index % self.cols as i32);
        ap.bed_idx = index;
        ap.row = row;
        ap.col = col;
        let (stride_x, stride_y) = self.stride();
        // `translation -= scaled<double>(...)`: the difference truncates to `coord_t`.
        let x = (ap.translation.x() as f64 - stride_x * f64::from(col) / SCALING_FACTOR) as i64;
        let y = (ap.translation.y() as f64 + stride_y * f64::from(row) / SCALING_FACTOR) as i64;
        ap.translation = Point::new(x, y);
    }

    /// `postprocess_bed_index_for_current_plate` (`PartPlate.cpp:7913-7928`): the first bed is
    /// the current plate, any other is the plate past the last one.
    pub(in crate::plater) fn bed_index_for_current_plate(
        &self,
        ap: &mut ArrangePolygon,
        current: usize,
    ) {
        ap.bed_idx = match ap.bed_idx {
            UNARRANGED => UNARRANGED,
            0 => current as i32,
            _ => self.plates.len() as i32,
        };
    }

    /// `clear(false, false, true, plate_index)` (`PartPlate.cpp:6454-6501`): every plate is
    /// cleared, `PartPlate::clear(false)` included (`:4173-4176`), so no plate holds an instance.
    pub(in crate::plater) fn clear_instances(&mut self) {
        for plate in &mut self.plates {
            plate.instances.clear();
        }
        self.unprintable.instances.clear();
    }

    /// `postprocess_bed_index_for_selected` (`PartPlate.cpp:7833-7879`): the item's bed steps
    /// past each locked plate, and plates are created until that bed exists.
    pub(in crate::plater) fn create_plates_for(&mut self, ap: &mut ArrangePolygon) {
        if ap.bed_idx == -1 {
            return;
        }
        for (index, plate) in self.plates.iter().enumerate() {
            if plate.locked {
                ap.bed_idx += 1;
            } else if ap.bed_idx as usize <= index {
                return;
            }
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
            locked: false,
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

    /// `reload_all_objects(true, plate_index)` (`PartPlate.cpp:7498-7546`) for any plate index:
    /// each instance on the first plate it reaches, else on the unprintable plate.
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

/// `preprocess_exclude_areas`'s resolved-region branch (`PartPlate.cpp:7725-7761`): each collision
/// volume of the active nozzle groups (the first alone when Shared) becomes a blocker on every
/// plate, with its z range and nozzle (-1 when Shared).
fn bed_exclusion_blockers(
    config: &DynamicPrintConfig,
    num_plates: i32,
    inflation: i64,
) -> Vec<ArrangePolygon> {
    if !has_bed_exclude_volumes(config) {
        return Vec::new();
    }
    let shared = active_bed_exclude_volume_mode(config) == BedExcludeVolumeMode::Shared;
    let groups = get_bed_excluded_regions_by_extruder(config);
    let extruder_count = if shared {
        groups.len().min(1)
    } else {
        groups.len()
    };
    let mut blockers = Vec::new();
    for (extruder_id, regions) in groups.iter().take(extruder_count).enumerate() {
        let volumes = regions.iter().enumerate().filter(|(_, region)| {
            region.is_collision_volume()
                && region.polygon.points().len() >= 3
                && !(region.has_z_range && region.z_max <= region.z_min + EPSILON)
        });
        for (region_id, region) in volumes {
            let contour = bed_exclusion_arrange_polygon(&region.polygon);
            blockers.extend((0..num_plates).map(|bed| ArrangePolygon {
                poly: ExPolygon::new(contour.clone(), Vec::new()),
                is_virt_object: true,
                is_bed_exclusion: true,
                bed_exclusion_extruder_id: if shared { -1 } else { extruder_id as i32 },
                bed_idx: bed,
                height: (region.z_max - region.z_min).max(0.0),
                z_min: region.z_min,
                z_max: region.z_max,
                has_z_range: region.has_z_range,
                name: format!("BedExclusionVolume{extruder_id}_{region_id}"),
                inflation,
                ..ArrangePolygon::default()
            }));
        }
    }
    blockers
}
