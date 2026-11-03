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

    /// The plate menu's "Delete All" (`Selection::remove_curr_plate`, `Selection.cpp:507-525` at
    /// v2.5.0-beta `345efec64`; see `docs/architecture/web-plater-plate-delete-all.md`) on the
    /// current plate.
    pub fn delete_plate_objects(&mut self) -> Result<(), SliceError> {
        // `get_volume_idxs_from_plate` (`Selection.cpp:2256`): every instance the plate contains
        // or overlaps, whole.
        let mut selected = std::collections::BTreeMap::<usize, Vec<usize>>::new();
        for key in super::session::keys(&self.project) {
            let bbox = super::selection::instance_bounding_box(&self.project, key);
            if self.plates.holds_or_reaches(self.current_plate, key, &bbox) {
                selected.entry(key.0).or_default().push(key.1);
            }
        }
        // `erase()` (`:1893-1996`): an object with every instance selected goes whole, else its
        // selected instances go; `delete_from_model_and_list` walks the items from the highest
        // object and instance index down (`GUI_ObjectList.hpp:86-91`, `.cpp:4490-4558`).
        for (object, instances) in selected.into_iter().rev() {
            if instances.len() == self.project.objects()[object].instances().len() {
                self.delete_object(object)?;
            } else {
                instances
                    .into_iter()
                    .rev()
                    .try_for_each(|instance| self.delete_instance(object, instance))?;
            }
        }
        Ok(())
    }

    /// The Edit menu's "Delete All" (`Plater::priv::delete_all_objects_from_model`,
    /// `Plater.cpp:10793-10821`; see `docs/architecture/web-plater-delete-all.md`). `InvalidInput`
    /// without objects, where `MainFrame::can_delete_all` (`MainFrame.cpp:2128-2131`) is false.
    pub fn delete_all_objects(&mut self) -> Result<(), SliceError> {
        if self.project.objects().is_empty() {
            return Err(SliceError::InvalidInput(
                "there is no object to delete".to_owned(),
            ));
        }
        self.plates.clear();
        self.project.delete_all_objects();
        self.arrange_orders.clear();
        Ok(())
    }

    /// "Delete" with one instance selected (`Plater::remove_selected` → `Selection::erase`,
    /// `Selection.cpp:1893-1996` at v2.5.0-beta `345efec64`; see
    /// `docs/architecture/web-plater-delete-instance.md`). `InvalidInput` for no such instance.
    pub fn delete_instance(&mut self, object: usize, instance: usize) -> Result<(), SliceError> {
        self.check(object, instance)?;
        let item = &self.project.objects()[object];
        // A lone instance is `is_single_full_object`: the object goes (`Selection.cpp:1898-1899`).
        if item.instances().len() == 1 {
            return self.delete_object(object);
        }
        // `del_subobject_from_object` (`GUI_ObjectList.cpp:3065, 3070`): `delete_instance` and
        // `changed_object`. No plate is notified: every plate keeps its pairs, and the last
        // one now names no instance.
        let id = (item.id(), item.instances()[instance].instance_id());
        self.project.delete_instance(object, instance);
        self.arrange_orders.remove(&id);
        Ok(())
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
