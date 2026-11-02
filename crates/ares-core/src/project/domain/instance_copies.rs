//! The model's objects and instances edited by the plater's "Add instance" / "Remove instance"
//! (`ModelObject::add_instance()` / `delete_last_instance`, `Model.cpp:1554-1563, 1591-1594` at
//! v2.5.0-beta `345efec64`) and "Delete" (`Model::delete_object`, `:672-679`). See
//! `docs/architecture/web-plater-instance-copies.md` and `web-plater-delete-object.md`.

use super::plater_edits::object_offset;
use super::{Project, ProjectInstance};
use crate::Transform3d;

impl Project {
    /// Appends a default instance (`new ModelInstance(this)`: printable) with `transform` and
    /// `identify_id` as its export identity. Its imported copy keeps the object's frame offset,
    /// instance 0's, as `set_instance_transform` keeps every instance's.
    pub(crate) fn add_instance(&mut self, object: usize, transform: Transform3d, identify_id: u32) {
        let object = &mut self.objects[object];
        let first = &object.instances[0];
        let frame = object
            .import_instances
            .iter()
            .find(|imported| imported.instance_id == first.instance_id)
            .map(|imported| object_offset(first.transform, imported.transform));
        let instance_id = object.instances.len() as u32;
        let instance = ProjectInstance::new(
            [object.id, instance_id, identify_id],
            true,
            false,
            transform,
        );
        if let Some(frame) = frame {
            object.import_instances.push(ProjectInstance {
                transform: transform.translated(frame),
                ..instance.clone()
            });
        }
        object.instances.push(instance);
    }

    /// `Model::delete_object(idx)`.
    pub(crate) fn delete_object(&mut self, object: usize) {
        self.objects.remove(object);
    }

    /// "Delete All" (`delete_all_objects_from_model`, `Plater.cpp:10813, 10821`):
    /// `model.clear_objects()` and `model.plates_custom_gcodes.clear()`.
    pub(crate) fn delete_all_objects(&mut self) {
        self.objects.clear();
        self.plates_custom_gcodes.clear();
    }

    /// `delete_last_instance`, with its imported copy.
    pub(crate) fn remove_last_instance(&mut self, object: usize) {
        let object = &mut self.objects[object];
        let removed = object.instances.pop().expect("an instance to remove");
        object
            .import_instances
            .retain(|imported| imported.instance_id != removed.instance_id);
    }
}
