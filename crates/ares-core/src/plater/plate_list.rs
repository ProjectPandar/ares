//! `PartPlateList` (`slic3r/GUI/PartPlate.cpp`): the plate grid, the
//! instances each plate holds, and the unprintable plate beside the grid.

use std::collections::BTreeSet;

use crate::project::{compute_colum_count, plate_origin_at};

#[path = "plate_list/arrange.rs"]
mod arrange;
#[path = "plate_list/extruders.rs"]
mod extruders;
pub(super) use extruders::volume_extruders;
#[path = "plate_list/tower_default.rs"]
mod tower_default;
#[path = "plate_list/wipe_tower.rs"]
mod wipe_tower;
pub(super) use wipe_tower::{Relocation, Towers};

/// `(object index, instance index)`, the key of `PartPlate::obj_to_instance_set`.
pub(super) type InstanceKey = (usize, usize);

/// A world bounding box `(min, max)`.
pub(super) type BoundingBox = ([f64; 3], [f64; 3]);

/// `BuildVolume::SceneEpsilon` (`BuildVolume.hpp:106`, `EPSILON`, `libslic3r.h:52`).
const SCENE_EPSILON: f64 = 1e-4;

/// The model the plate list reads and moves (`PartPlate::m_model`).
pub(super) trait PlateModel {
    /// Every instance in `Model::objects` / `instances` order.
    fn keys(&self) -> Vec<InstanceKey>;
    /// `ModelObject::instance_convex_hull_bounding_box` (`Model.cpp:1623-1638`).
    fn convex_hull_bounding_box(&self, key: InstanceKey) -> BoundingBox;
    /// `ModelInstance::set_offset(get_offset() + offset)`.
    fn translate(&mut self, key: InstanceKey, offset: [f64; 2]);
}

#[derive(Clone, Debug)]
pub(super) struct Plate {
    pub(super) origin: [f64; 2],
    pub(super) instances: BTreeSet<InstanceKey>,
    /// The loaded plate this one came from (its documents and per-plate data).
    pub(super) source: Option<usize>,
}

#[derive(Clone, Debug)]
pub(super) struct PlateList {
    pub(super) plates: Vec<Plate>,
    pub(super) unprintable: Plate,
    /// `m_plate_width`, `m_plate_depth`, `m_plate_height`.
    size: (i32, i32),
    height: i32,
    /// The printable area's bounding box before the plate offset (`m_shape`).
    shape: ([f64; 2], [f64; 2]),
    cols: usize,
}

impl PlateList {
    pub(super) fn new(
        plates: Vec<BTreeSet<InstanceKey>>,
        size: (i32, i32),
        height: i32,
        shape: ([f64; 2], [f64; 2]),
    ) -> Self {
        let cols = compute_colum_count(plates.len());
        let plates = plates
            .into_iter()
            .enumerate()
            .map(|(index, instances)| Plate {
                origin: origin(index, cols, size),
                instances,
                source: Some(index),
            })
            .collect::<Vec<_>>();
        let mut list = Self {
            unprintable: Plate {
                origin: [0.0; 2],
                instances: BTreeSet::new(),
                source: None,
            },
            plates,
            size,
            height,
            shape,
            cols,
        };
        list.unprintable.origin = list.origin_for_unprintable();
        list
    }

    /// `m_plate_width`, `m_plate_depth`.
    pub(super) fn size(&self) -> (i32, i32) {
        self.size
    }

    pub(super) fn plate_origin(&self, index: usize) -> [f64; 2] {
        self.plates[index].origin
    }

    /// `compute_origin_for_unprintable` (`PartPlate.cpp:3942-3949`).
    fn origin_for_unprintable(&self) -> [f64; 2] {
        let count = self.plates.len();
        let max_count = self.cols * self.cols;
        if count == max_count {
            origin(max_count + self.cols - 1, self.cols + 1, self.size)
        } else {
            origin(count, self.cols, self.size)
        }
    }

    /// `PartPlate::get_build_volume` (`PartPlate.cpp:3280-3299`) of a plate at `origin`.
    fn plate_box(&self, origin: [f64; 2]) -> BoundingBox {
        let (min, max) = self.shape;
        (
            [
                min[0] + origin[0] - SCENE_EPSILON,
                min[1] + origin[1] - SCENE_EPSILON,
                -SCENE_EPSILON,
            ],
            [
                max[0] + origin[0] + SCENE_EPSILON,
                max[1] + origin[1] + SCENE_EPSILON,
                f64::from(self.height) + SCENE_EPSILON,
            ],
        )
    }

    /// `PartPlate::intersect_instance` (`PartPlate.cpp:2665-2688`); the
    /// unprintable plate tests `is_left_top_of` (`:2690-2708`).
    fn intersects(&self, plate: &Plate, printable: bool, bbox: &BoundingBox) -> bool {
        if !printable {
            return plate.origin[0] <= bbox.0[0] && plate.origin[1] >= bbox.0[1];
        }
        let (min, max) = self.plate_box(plate.origin);
        (0..3).all(|axis| min[axis] < bbox.1[axis] && max[axis] > bbox.0[axis])
    }

    /// `PartPlateList::notify_instance_update` (`PartPlate.cpp:5039-5170`)
    /// for a moved or rotated instance.
    pub(super) fn notify_instance_update(&mut self, model: &impl PlateModel, key: InstanceKey) {
        let bbox = model.convex_hull_bounding_box(key);
        if let Some(index) = self
            .plates
            .iter()
            .position(|plate| plate.instances.contains(&key))
        {
            if self.intersects(&self.plates[index], true, &bbox) {
                return;
            }
            self.plates[index].instances.remove(&key);
        } else if self.unprintable.instances.contains(&key) {
            if self.intersects(&self.unprintable, false, &bbox) {
                return;
            }
            self.unprintable.instances.remove(&key);
        }
        if let Some(index) =
            (0..self.plates.len()).find(|index| self.intersects(&self.plates[*index], true, &bbox))
        {
            self.plates[index].instances.insert(key);
        } else if self.intersects(&self.unprintable, false, &bbox) {
            self.unprintable.instances.insert(key);
        }
    }

    /// `PartPlate::set_pos_and_size` (`PartPlate.cpp:2338-2398`) for an
    /// unchanged size: with `move_instances`, every instance follows the
    /// plate by the integer-truncated origin change.
    fn set_origin(
        plate: &mut Plate,
        origin: [f64; 2],
        move_instances: bool,
        model: &mut impl PlateModel,
    ) {
        if plate.origin == origin {
            return;
        }
        if move_instances {
            let offset = [0, 1].map(|axis| f64::from((origin[axis] - plate.origin[axis]) as i32));
            for key in &plate.instances {
                model.translate(*key, offset);
            }
        }
        plate.origin = origin;
    }

    /// `update_all_plates_pos_and_size` (`PartPlate.cpp:4872-4890`).
    fn update_all_plates_pos(
        &mut self,
        move_instances: bool,
        move_unprintable: bool,
        model: &mut impl PlateModel,
    ) {
        for index in 0..self.plates.len() {
            let target = origin(index, self.cols, self.size);
            Self::set_origin(&mut self.plates[index], target, move_instances, model);
        }
        let target = self.origin_for_unprintable();
        Self::set_origin(&mut self.unprintable, target, move_unprintable, model);
    }

    /// `PartPlateList::create_plate(true)` (`PartPlate.cpp:4407-4480`) and
    /// `construct_objects_list_for_new_plate` (`:5310-5360`).
    pub(super) fn create_plate(&mut self, model: &mut impl PlateModel) {
        let new_index = self.plates.len();
        let cols = compute_colum_count(new_index + 1);
        let old_cols = compute_colum_count(new_index);
        self.plates.push(Plate {
            origin: origin(new_index, cols, self.size),
            instances: BTreeSet::new(),
            source: None,
        });
        self.cols = compute_colum_count(self.plates.len());
        if old_cols != cols {
            self.update_all_plates_pos(true, false, model);
        } else {
            let target = self.origin_for_unprintable();
            Self::set_origin(&mut self.unprintable, target, false, model);
        }
        self.unprintable.instances.clear();
        for key in model.keys() {
            if self.plates[..new_index]
                .iter()
                .any(|plate| plate.instances.contains(&key))
            {
                continue;
            }
            let bbox = model.convex_hull_bounding_box(key);
            if self.intersects(&self.plates[new_index], true, &bbox) {
                self.plates[new_index].instances.insert(key);
            } else if self.intersects(&self.unprintable, false, &bbox) {
                self.unprintable.instances.insert(key);
            }
        }
    }

    /// `PartPlateList::delete_plate` (`PartPlate.cpp:4554-4660`): the plate's
    /// instances move with it to the unprintable position, later plates move
    /// up, and the deleted instances go to the last plate or the unprintable one.
    pub(super) fn delete_plate(&mut self, index: usize, model: &mut impl PlateModel) -> bool {
        if index >= self.plates.len() || self.plates.len() <= 1 {
            return false;
        }
        let cols = compute_colum_count(self.plates.len() - 1);
        let old_cols = compute_colum_count(self.plates.len());
        let mut deleted = self.plates.remove(index);
        self.cols = compute_colum_count(self.plates.len());
        let target = self.origin_for_unprintable();
        Self::set_origin(&mut deleted, target, true, model);
        for later in index..self.plates.len() {
            let target = origin(later, self.cols, self.size);
            Self::set_origin(&mut self.plates[later], target, true, model);
        }
        if old_cols != cols {
            self.update_all_plates_pos(true, true, model);
        } else {
            let target = self.origin_for_unprintable();
            Self::set_origin(&mut self.unprintable, target, true, model);
        }
        let last = self.plates.len() - 1;
        for key in deleted.instances {
            let bbox = model.convex_hull_bounding_box(key);
            if self.intersects(&self.plates[last], true, &bbox) {
                self.plates[last].instances.insert(key);
            } else {
                self.unprintable.instances.insert(key);
            }
        }
        true
    }
}

fn origin(index: usize, cols: usize, size: (i32, i32)) -> [f64; 2] {
    let (x, y) = plate_origin_at(index, cols, size);
    [x, y]
}
