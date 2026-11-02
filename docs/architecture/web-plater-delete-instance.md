# "Delete" of one instance on the plater session

Every `file:line` is at OrcaSlicer v2.5.0-beta `345efec64`. "Delete" with one instance of a multi-instance object selected removes that instance only. The session can delete a whole object (`web-plater-delete-object.md`) and the last instance (`web-plater-number-of-instances.md`), but not an instance in the middle. The plate menu's "Delete All" (`remove_curr_plate_all`, deferred in `web-plater-delete-all.md`) needs this path.

## Upstream boundary
- **Entry.** `Plater::remove_selected` (`Plater.cpp:19270-19287`), enabled by `can_delete` (`:15039-15042`: a selection that is not the wipe tower), calls the canvas's `delete_selected`, which calls `Selection::erase` (`Selection.cpp:1893-1996`). The selection decides the path:
  - one instance of a one-instance object is `is_single_full_object`, which deletes the object (the ported delete-object path);
  - one instance of a multi-instance object is `is_single_full_instance`, which calls `ObjectList::delete_from_model_and_list(itInstance, obj, inst)` (`GUI_ObjectList.cpp:4461-4488`).
- **`del_subobject_from_object(obj, inst, itInstance)`** (`GUI_ObjectList.cpp:2998-3073`).
  - It refuses the last instance with an error dialog (`:3060-3064`). The selection never sends that case, because a lone instance is a full object.
  - Otherwise it calls `ModelObject::delete_instance(idx)` (`Model.cpp:1583-1588`): the instance leaves the vector and the later instances move down one index.
  - Then `changed_object` (`Plater.cpp:22240-22268`): `ensure_on_bed` and a scene reload.
- **The plates are not told.**
  - Nothing on this path calls `notify_instance_removed(obj, inst)`. That function (`PartPlate.cpp:7396-7455`) would only erase the pair and never renumbers later instances; its instance-level callers are the last-instance paths.
  - `reload_scene` (`GLCanvas3D.cpp:3170-3827`) only reads the plate list, and the canvas notifies plates on moves only (`:6082`).
  - So every plate's `obj_to_instance_set` keeps its pairs. With n instances before the delete:
    - `(obj, n-1)` now names no instance;
    - each `(obj, k)` with k ≥ idx keeps its plate but names the instance that moved down into slot k.
- **Readers guard the dangling pair.**
  - Plate geometry range-checks it (`PartPlate::get_objects_bounding_box`, `PartPlate.cpp:4749`).
  - `store_to_3mf_structure` copies every pair (`PartPlate.cpp:8458-8463`).
  - The 3MF writer emits the dangling pair as a `<model_instance>` with `identify_id` 0 (`bbs_3mf.cpp:8580-8600`) and skips it in the plate's G-code object list (`:8843-8846`).
- **No UB on this path.** Every index is bounds-checked. The `decrease_instances` use-after-free (`Plater.cpp:19354-19355`) is not on this path and is not ported.

## Ares destination
- **`Project::delete_instance(object, instance)`**: `ModelObject::delete_instance`, with its imported copy, as `remove_last_instance` does.
- **`PlaterSession::delete_instance(object, instance)`**:
  - `InvalidInput` for a missing object or instance;
  - a one-instance object goes to `delete_object`, as the selection routes it;
  - otherwise the project delete. Every plate keeps its pairs literally, the dangling `(object, n-1)` included. The removed instance's arrange order goes too.
- **`PlaterEdits`.** An object's instances become its ordered surviving loaded instance ids plus its copies, replacing `kept`: after a middle delete the survivors are no longer a prefix.
- **Plate readers mirror upstream's range checks.** Every Rust reader that indexes the project by a plate pair skips a pair whose instance no longer exists, where the upstream reader range-checks it (`PartPlate.cpp:4749`, `bbs_3mf.cpp:8843-8846`). The exported plate map writes it as upstream's writer does: object id, the stale instance id, `identify_id` 0 (`bbs_3mf.cpp:8580-8600`).
- **wasm:** `deleteInstance(object, instance)`.
- **Web (object list).**
  - An object with more than one instance shows an "Instances" node with "Instance N" items (`ObjectDataViewModel.cpp:136-145`).
  - An instance item is selectable. Delete/Backspace and the object menu's "Delete" on it delete that instance. On the object row they still delete the object.

## Decision (review)
- **Cross-plate objects are ported literally, option (a).** When an object's instances sit on different plates, a plate keeps `(obj, k)` after a middle delete while the instance now in slot k is on another plate. This is defined behaviour, and Native slicing the GUI export witnesses it.
- **The dangling `(obj, n-1)` pair is kept,** since the 3MF export bytes are a public output. No reader deviates.

## Included and deferred
- **Included:** the single-instance delete and the object-list instance items.
- **Deferred:**
  - multi-selection deletes (`is_multiple_full_instance`, mixed selections);
  - the `[P%d]Instance %d` plate prefix (`ObjectDataViewModel.cpp:400-405`);
  - selecting an instance in the 3D view;
  - the plate menu's "Delete All", which comes next on top of this.

## Oracle and RED
In `plater_delete_instance.rs`, the session-edit Native slice of the GUI export:
1. **Same plate.** `combined-brim.3mf` (object 2 has two instances; object 3 has one). Delete instance 0 of object 2: one instance remains, on plate 1, and the plate slices like Native slicing the export.
2. **Lone instance.** Deleting object 3's only instance deletes the object, as `delete_object` does.
3. **Bounds.** A missing object or instance is `InvalidInput`.
4. **Cross-plate.** Object 2's instance 1 is moved onto a second plate, then instance 0 is deleted. Plate 1 keeps `(0, 0)`, now the instance on plate 2's area. Plate 2 keeps `(0, 1)`, now dangling. Each plate slices like Native slicing the export.
5. **The dangling pair.** After case 1, `plates()` still lists `(0, 1)` on plate 1. The exported plate map carries its `<model_instance>` with `instance_id` 1 and `identify_id` 0, as upstream's writer emits it.

Web E2E `plateDeleteInstance.spec.ts`: select "Instance 1" of the two-instance object, press Delete, then the list shows the object without an "Instances" node and the plate slices like Native.
