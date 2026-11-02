# Deleting an object on the plater session

Every `file:line` is at OrcaSlicer v2.5.0-beta `345efec64`. The plater's object menu "Delete" (Del on Windows, Backspace elsewhere) removes the selected object from the model. The session cannot remove an object today. It is also the missing piece behind `decrease_instances`' object removal, which `web-plater-instance-copies.md` defers.

## Upstream boundary
- **The menu item** is `MenuFactory::append_menu_item_delete` (`GUI_Factories.cpp:584-595`), labelled "Delete the selected object". It calls `Plater::remove_selected` (`Plater.cpp:19270-19287`), which needs `can_delete`: a non-empty selection that is not the wipe tower (`:15039-15042`).
- **The call chain** goes through `GLCanvas3D::delete_selected` to `Selection::erase` (`Selection.cpp:1893`). For a whole object, `Selection::erase` calls `ObjectList::delete_from_model_and_list(itObject, obj_idx)` (`GUI_ObjectList.cpp:4461`), then `del_object` (`:2812-2815`), then `Plater::priv::delete_object_from_model` (`Plater.cpp:10750-10791`).
- **`delete_object_from_model`:**
  - `Model::delete_object(idx)` (`Model.cpp:672-679`) erases the object from `Model::objects`.
  - Then `PartPlateList::notify_instance_removed(obj_idx, -1)` (`PartPlate.cpp:7396-7455`) removes each of the object's instances from its plate and from the unprintable plate. `update_object_index` then renumbers every later object's entries down by one.

- **After the deletion** (`Plater.cpp:10784-10788`): `update()` (`:8405-8441`) and `object_list_changed()` (`:10672-10696`) reset no plate or project state.
  - `update()` reloads the scene, whose tower re-clamp touches only shown towers (an emptied plate shows none), and reschedules background processing, so the slice results go stale.
  - `object_list_changed()` recomputes `can_slice`, false once `model.objects` is empty, which disables the slice buttons.
  - Deleting the last object therefore leaves the plates, their locks and the tower positions as they were. The web already marks every slice result stale on an edit (`applyLayout`).

## Ares destination
- **`PlaterSession::delete_object(object)`.** `InvalidInput` for no such object. Deleting the last object is allowed, as upstream allows it, which leaves an empty plate.
- **`Project::delete_object`.** The plate list removes the object's keys and shifts later objects' indices. The session's per-object state (the A8 kept-instance counts, the arrange orders) follows by object id.
- **`PlaterEdits`** gains the deleted object ids, applied first when slicing, so slicing and the export leave the object out.
- **wasm:** `deleteObject`.
- **Web:** the object menu's "Delete", with Delete and Backspace on the selected object.

## Included and deferred
- **Included:** deleting a whole object, from the menu and the keys, through slicing and the export.
- **Deferred:**
  - deleting a single instance (`itInstance`, `ModelObject::delete_instance` with instance renumbering);
  - deleting volumes and mixed selections;
  - "Delete all" (`delete_all_objects_from_model`, `Plater.cpp:10793`);
  - the cut-object warning (`delete_object_from_model`'s `obj->is_cut()` branch). Cut is not ported, so `is_cut()` is always false in the session and the branch is unreachable, not skipped;
  - `decrease_instances` removing the object, which stays with "Set number of instances";
  - the undo snapshot.

## Oracle and RED
- **The session-edit Native oracle**, as for the other plate edits (`assert_plate_like_native`). The test-side GUI export leaves out the deleted object's resource, its build items and its `model_settings.config` entry. Native slices that export, and the session's slice must match.
- **Cases** in `plater_delete_object.rs`:
  1. **Same plate:** the by-object cubes (`combined-brim.3mf`: object 2 with two instances, object 3 with one). Delete object 0, and plate 1 holds object 3's instance, now object 0. The plate slices like Native.
  2. **Another plate:** `two-plates.3mf`. Delete plate 1's cube; plate 1 is empty, and plate 2 holds the other cube as object 0. Plate 2 slices like Native.
  3. **Session-only:** deleting a missing object returns `InvalidInput`.
- **Web E2E** `plateDeleteObject.spec.ts`: delete a cube from the object menu, then "Slice plate", compared with the runtime Native oracle for the export.
- **Expected before GREEN:** cases 1-3 fail on the `todo!()` skeleton of `delete_object`.
