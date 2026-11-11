//! An undo/redo snapshot of the plater: what `UndoRedo::StackImpl::take_snapshot` saves of the
//! `Model` and the `PartPlateList` (`slic3r/Utils/UndoRedo.cpp:937-975`), and what
//! `load_snapshot` (`:1009-1089`) restores. The stack of snapshots is the caller's.

use std::collections::BTreeMap;

use super::PlaterSession;
use super::arrange_job::TowerPositions;
use super::plate_list::PlateList;
use crate::project::ModelSnapshot;

pub struct PlaterSnapshot {
    model: ModelSnapshot,
    plates: PlateList,
    arrange_orders: BTreeMap<(u32, u32), i32>,
    wipe_tower: Option<TowerPositions>,
    current_plate: usize,
}

impl PlaterSession {
    pub fn snapshot(&self) -> PlaterSnapshot {
        PlaterSnapshot {
            model: self.project.model_snapshot(),
            plates: self.plates.clone(),
            arrange_orders: self.arrange_orders.clone(),
            wipe_tower: self.wipe_tower.clone(),
            current_plate: self.current_plate,
        }
    }

    pub fn load_snapshot(&mut self, snapshot: &PlaterSnapshot) {
        self.project.load_model_snapshot(&snapshot.model);
        self.plates.clone_from(&snapshot.plates);
        self.arrange_orders.clone_from(&snapshot.arrange_orders);
        self.wipe_tower.clone_from(&snapshot.wipe_tower);
        self.current_plate = snapshot.current_plate;
    }
}
