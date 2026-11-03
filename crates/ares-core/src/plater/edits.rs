//! The edited model state a plater session hands to slicing: what the GUI's
//! project export writes for it (`bbs_3mf.cpp:8050-8073` build item
//! transforms, `:7918` plate map). Slicing loads the input and applies these
//! before plate selection.

use crate::Transform3d;

/// One plate of an edited plate list, in plate order.
#[derive(Clone, Debug)]
pub(crate) struct EditedPlate {
    /// The loaded plate whose per-plate documents this plate keeps.
    pub(crate) source: Option<usize>,
    /// `[object_id, instance_id, identify_id]` of each instance on the plate.
    pub(crate) instances: Vec<[u32; 3]>,
}

/// An object's instances after "Add instance" / "Remove instance" / "Delete": the loaded
/// instances it keeps, by instance id in order, then `copies` as
/// `([instance id, identify id], transformation)`, in order.
#[derive(Clone, Debug)]
pub(crate) struct InstanceChange {
    pub(crate) object_id: u32,
    pub(crate) kept: Vec<u32>,
    pub(crate) copies: Vec<([u32; 2], Transform3d)>,
}

/// Instance transformations and plate map of an edited project; empty
/// edits leave the loaded project unchanged.
#[derive(Clone, Debug, Default)]
pub struct PlaterEdits {
    /// By `(object_id, instance_id)`.
    pub(crate) transforms: Vec<((u32, u32), Transform3d)>,
    pub(crate) plates: Option<Vec<EditedPlate>>,
    /// The object ids in model order, which an arrange sorts (`PartPlate.cpp:8389`).
    pub(crate) object_order: Option<Vec<u32>>,
    /// `wipe_tower_x`/`wipe_tower_y` of the project config, one entry per plate.
    pub(crate) wipe_tower: Option<(Vec<f64>, Vec<f64>)>,
    /// The objects whose instances "Add instance" / "Remove instance" changed.
    pub(crate) instances: Vec<InstanceChange>,
    /// The ids of the objects "Delete" removed.
    pub(crate) deleted_objects: Vec<u32>,
}

impl PlaterEdits {
    pub(crate) const NONE: Self = Self {
        transforms: Vec::new(),
        plates: None,
        object_order: None,
        wipe_tower: None,
        instances: Vec::new(),
        deleted_objects: Vec::new(),
    };

    pub(crate) fn transform(&self, object_id: u32, instance_id: u32) -> Option<Transform3d> {
        self.transforms
            .iter()
            .find(|(key, _)| *key == (object_id, instance_id))
            .map(|(_, transform)| *transform)
    }
}
