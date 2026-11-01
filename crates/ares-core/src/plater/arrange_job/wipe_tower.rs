//! The arrange job's prime tower items (v2.5.0-beta `345efec64`): the tower each plate shows
//! (`GLCanvas3D::reload_scene`, `GLCanvas3D.cpp:3564-3663`; `load_wipe_tower_preview`,
//! `3DScene.cpp:987-1052`; `GLCanvas3D::get_wipe_tower_info`, `GLCanvas3D.cpp:6149-6185`;
//! `WipeTower::get_arrange_polygon`, `ArrangeJob.cpp:122-145`), `prepare_wipe_tower`
//! (`ArrangeJob.cpp:442-528`) with `estimate_wipe_tower_info` (`:416-430`), and the tower
//! position of a plate the arrange adds (`PartPlateList::create_plate`, `PartPlate.cpp:6644-6646`).

use std::collections::BTreeSet;

use super::{MAX_NUM_PLATES, needs_wipe_tower, prepare};
use crate::SliceError;
use crate::arrangement::{ArrangeParams, ArrangePolygon};
use crate::geometry::{ExPolygon, JoinType, Point, Polygon, offset_paths};
use crate::options::ProcessPrintSequence;
use crate::plater::PlaterSession;
use crate::plater::plate_list::{Relocation, Towers};
use crate::project_slice::gcode_emit::wipe_tower_estimate::config::{DynamicConfig, get_at};
use crate::project_slice::gcode_emit::wipe_tower_estimate::{
    auto_brim_by_height, first_layer_outline, resolve_wipe_tower_type,
};

/// `EPSILON` (`libslic3r.h`).
const EPSILON: f64 = 1e-4;

/// The stored tower positions, `wipe_tower_x` and `wipe_tower_y`.
pub(in crate::plater) type TowerPositions = (Vec<f64>, Vec<f64>);

/// `std::clamp(value, 0.0, hi)`.
fn clamp(value: f64, hi: f64) -> f64 {
    if value < 0.0 {
        0.0
    } else if hi < value {
        hi
    } else {
        value
    }
}

/// `ConfigOptionFloats::set_at`: grown with copies of the first value.
fn set_at(values: &mut Vec<f64>, index: usize, value: f64) {
    if values.len() <= index {
        let first = values[0];
        values.resize(index + 1, first);
    }
    values[index] = value;
}

/// `GLCanvas3D::WipeTowerInfo` of a plate's tower volume: its box with the brim and the
/// position clamped onto the plate.
#[derive(Clone, Copy, Debug)]
pub(super) struct ShownTower {
    bb: ([f64; 2], [f64; 2]),
    pos: [f64; 2],
}

impl ShownTower {
    /// `WipeTower::get_arrange_polygon` through `get_wipetower_arrange_poly`: a fixed virtual
    /// item on bed 0 with a raised priority.
    pub(super) fn arrange_polygon(&self) -> ArrangePolygon {
        let scaled = |value: f64| (value / 1e-6) as i64;
        let (min, max) = self.bb;
        let contour = Polygon::new(vec![
            Point::new(scaled(min[0]), scaled(min[1])),
            Point::new(scaled(max[0]), scaled(min[1])),
            Point::new(scaled(max[0]), scaled(max[1])),
            Point::new(scaled(min[0]), scaled(max[1])),
        ]);
        ArrangePolygon {
            poly: ExPolygon::new(contour, Vec::new()),
            translation: Point::new(scaled(self.pos[0]), scaled(self.pos[1])),
            name: "WipeTower".to_owned(),
            is_virt_object: true,
            is_wipe_tower: true,
            priority: 1,
            bed_idx: 0,
            ..ArrangePolygon::default()
        }
    }
}

/// The 2D box of `load_wipe_tower_preview`'s shell, its height last: the `float` cube and, with a
/// brim, the brim slab under the first-layer outline grown by it. `None` where no volume loads.
fn preview_box(
    config: &DynamicConfig,
    (width, depth, height): (f32, f32, f32),
    brim: f32,
) -> Option<([f32; 2], [f32; 2], f32)> {
    if depth < 0.01 {
        return None;
    }
    let height = if height == 0.0 { 0.1 } else { height };
    let (mut min, mut max, mut top) = ([0.0_f32; 2], [width, depth], height);
    if brim > 0.0 {
        let outline = first_layer_outline(
            config,
            resolve_wipe_tower_type(config),
            (f64::from(width), f64::from(depth), f64::from(height)),
        );
        let delta = (brim / 1e-6_f32) as i64 as f32;
        let grown = offset_paths(std::slice::from_ref(&outline), delta, JoinType::Miter, 3.0)
            .expect("brim outline");
        let slab = grown.first().unwrap_or(&outline);
        // `its_make_rib_brim`: no slab under an outline smaller than `scaled(EPSILON)`.
        if slab.area() >= (EPSILON / 1e-6).trunc() {
            for point in slab.points() {
                let (x, y) = (point.x() as f32 * 1e-6_f32, point.y() as f32 * 1e-6_f32);
                min = [min[0].min(x), min[1].min(y)];
                max = [max[0].max(x), max[1].max(y)];
            }
            top = top.max(0.2);
        }
    }
    Some((min, max, top))
}

impl PlaterSession {
    /// `PartPlate::printable_instance_size`: the plate's printable instances not outside it.
    fn printable_instance_size(&self, plate: usize) -> Result<usize, SliceError> {
        let mut size = 0;
        for &key in &self.plates.plates[plate].instances {
            let printable = self.project.objects()[key.0].instances()[key.1].printable();
            if printable && self.plates.contains_totally(plate, key, &self.project)? {
                size += 1;
            }
        }
        Ok(size)
    }

    /// The towers `reload_scene` shows, per plate, after it re-clamps each stored position.
    pub(super) fn shown_towers(
        &self,
        config: &DynamicConfig,
        positions: &mut TowerPositions,
    ) -> Result<Vec<Option<ShownTower>>, SliceError> {
        let count = self.plates.plates.len();
        let mut shown = vec![None; count];
        let filaments = config.option("filament_colour").strings().len();
        let need = config.option("timelapse_type").key() == "1"
            || config.option("enable_wrapping_detection").bool();
        // Belt printers are deferred (no `belt_printer` key); the plater shows a G-code 3MF never.
        if !(config.option("enable_prime_tower").bool() && (need || filaments > 1)) {
            return Ok(shown);
        }
        let global_by_object = config.option("print_sequence").key() == "by object";
        for (plate, slot) in shown.iter_mut().enumerate() {
            let by_object = prepare::plate_sequence(&self.project, &self.plates, plate)
                .map_or(global_by_object, |sequence| {
                    sequence == ProcessPrintSequence::ByObject
                });
            if by_object && self.printable_instance_size(plate)? != 1 {
                continue;
            }
            if self.plates.plates[plate].instances.is_empty() {
                continue;
            }
            let towers = Towers {
                plates: &self.plates,
                project: &self.project,
                config,
                positions: (&positions.0, &positions.1),
            };
            let footprint = towers.footprint(plate, 0, false)?;
            if footprint.depth <= 0.0 {
                continue;
            }
            let (x, y) = (
                get_at(&positions.0, plate) as f32,
                get_at(&positions.1, plate) as f32,
            );
            let mut relocation = Relocation::default();
            let (_, clamped, _) = towers.polygon((plate, plate), 0, Some(&mut relocation))?;
            if f64::from((x - clamped[0] as f32).abs()) > EPSILON
                || f64::from((y - clamped[1] as f32).abs()) > EPSILON
            {
                set_at(&mut positions.0, plate, f64::from(clamped[0] as f32));
                set_at(&mut positions.1, plate, f64::from(clamped[1] as f32));
            }
            let size = (
                footprint.width as f32,
                footprint.depth as f32,
                footprint.height as f32,
            );
            let Some((min, max, top)) = preview_box(config, size, footprint.brim_width as f32)
            else {
                continue;
            };
            // `get_wipe_tower_info`: the box offset by the brim option twice, the stored position
            // clamped onto the plate.
            let mut brim = config.option("prime_tower_brim_width").float() as f32;
            if brim < 0.0 {
                brim = auto_brim_by_height(top);
            }
            let mut bb = (
                [f64::from(min[0]), f64::from(min[1])],
                [f64::from(max[0]), f64::from(max[1])],
            );
            for _ in 0..2 {
                let delta = f64::from(brim);
                bb = (
                    [bb.0[0] - delta, bb.0[1] - delta],
                    [bb.1[0] + delta, bb.1[1] + delta],
                );
            }
            let (plate_width, plate_depth) = self.plates.size();
            let pos = [
                clamp(
                    get_at(&positions.0, plate),
                    f64::from(plate_width) - (bb.1[0] - bb.0[0]),
                ),
                clamp(
                    get_at(&positions.1, plate),
                    f64::from(plate_depth) - (bb.1[1] - bb.0[1]),
                ),
            ];
            *slot = Some(ShownTower { bb, pos });
        }
        Ok(shown)
    }

    /// `prepare_wipe_tower` for "Arrange all": each bed's shown tower, else the estimate when one
    /// is needed.
    pub(super) fn prepare_wipe_tower(
        &self,
        config: &DynamicConfig,
        (positions, shown): (&TowerPositions, &[Option<ShownTower>]),
        params: &ArrangeParams,
        selected: &[ArrangePolygon],
    ) -> Result<Vec<ArrangePolygon>, SliceError> {
        let mut items = Vec::new();
        if !config.option("enable_prime_tower").bool() || params.is_seq_print {
            return Ok(items);
        }
        let need = needs_wipe_tower(self.project.settings(), params, selected);
        let count = self.plates.plates.len();
        let mut extruder_ids = BTreeSet::new();
        for plate in 0..count {
            extruder_ids.extend(self.plates.plate_extruders(plate, &self.project, config)?);
        }
        let towers = Towers {
            plates: &self.plates,
            project: &self.project,
            config,
            positions: (&positions.0, &positions.1),
        };
        for bed in 0..MAX_NUM_PLATES as usize {
            let valid = bed.min(count - 1);
            let item = match shown.get(bed).copied().flatten() {
                Some(tower) => Some(tower.arrange_polygon()),
                None if need => Some(towers.polygon((valid, bed), extruder_ids.len(), None)?.0),
                None => None,
            };
            if let Some(mut item) = item {
                item.bed_idx = bed as i32;
                items.push(item);
            }
        }
        Ok(items)
    }

    /// `create_plate`'s `set_default_wipe_tower_pos_for_plate(new_index, true)` for each plate from
    /// `first` on: the stored positions resized to the plates, the new plates' positions set.
    pub(in crate::plater) fn place_new_towers(
        &self,
        config: &DynamicConfig,
        positions: &mut TowerPositions,
        first: usize,
    ) -> Result<(), SliceError> {
        for plate in first..self.plates.plates.len() {
            let count = plate + 1;
            for values in [&mut positions.0, &mut positions.1] {
                let front = values[0];
                values.resize(count, front);
            }
            let towers = Towers {
                plates: &self.plates,
                project: &self.project,
                config,
                positions: (&positions.0, &positions.1),
            };
            let [x, y] = towers.default_position(plate)?;
            set_at(&mut positions.0, plate, f64::from(x as f32));
            set_at(&mut positions.1, plate, f64::from(y as f32));
        }
        Ok(())
    }
}
