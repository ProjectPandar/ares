//! The object menu's "Delete" of a whole object (`Plater::priv::delete_object_from_model`,
//! `Plater.cpp:10750-10791` at v2.5.0-beta `345efec64`). See
//! `docs/architecture/web-plater-delete-object.md`.

use super::PlaterSession;
use crate::SliceError;

impl PlaterSession {
    /// `delete_object_from_model(object)`: `Model::delete_object`, then
    /// `notify_instance_removed(object, -1)`. The cut-object warning cannot apply: no session object
    /// is cut. `InvalidInput` for no such object.
    pub fn delete_object(&mut self, object: usize) -> Result<(), SliceError> {
        let id = self
            .project
            .objects()
            .get(object)
            .ok_or_else(|| SliceError::InvalidInput(format!("there is no object {object}")))?
            .id();
        self.project.delete_object(object);
        self.plates.notify_object_removed(object);
        self.arrange_orders
            .retain(|&(object_id, _), _| object_id != id);
        Ok(())
    }

    /// The Edit menu's "Delete All" (`Plater::priv::delete_all_objects_from_model`,
    /// `Plater.cpp:10793-10821`; see `docs/architecture/web-plater-delete-all.md`). `InvalidInput`
    /// without objects, where `MainFrame::can_delete_all` (`MainFrame.cpp:2128-2131`) is false.
    pub fn delete_all_objects(&mut self) -> Result<(), SliceError> {
        todo!()
    }

    /// The loaded objects `delete_object` removed, by id.
    pub(super) fn deleted_objects(&self) -> Vec<u32> {
        self.loaded_instances
            .keys()
            .copied()
            .filter(|id| {
                !self
                    .project
                    .objects()
                    .iter()
                    .any(|object| object.id() == *id)
            })
            .collect()
    }
}
