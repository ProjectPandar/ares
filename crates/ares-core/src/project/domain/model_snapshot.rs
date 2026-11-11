//! The `Model` part of an undo/redo snapshot: `UndoRedo::StackImpl::take_snapshot` saves the
//! model (`slic3r/Utils/UndoRedo.cpp:937-975`) and `load_snapshot` puts it back (`:1009-1089`).
//! Plater edits change the objects and, through "Delete All", the per-plate custom G-codes.

use std::collections::BTreeMap;

use super::{Project, ProjectObject};
use crate::project::CustomGCodeInfo;

pub(crate) struct ModelSnapshot {
    objects: Vec<ProjectObject>,
    plates_custom_gcodes: BTreeMap<i32, CustomGCodeInfo>,
}

impl Project {
    pub(crate) fn model_snapshot(&self) -> ModelSnapshot {
        ModelSnapshot {
            objects: self.objects.clone(),
            plates_custom_gcodes: self.plates_custom_gcodes.clone(),
        }
    }

    pub(crate) fn load_model_snapshot(&mut self, snapshot: &ModelSnapshot) {
        self.objects.clone_from(&snapshot.objects);
        self.plates_custom_gcodes
            .clone_from(&snapshot.plates_custom_gcodes);
    }
}
