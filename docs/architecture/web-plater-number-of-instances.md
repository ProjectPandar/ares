# "Set number of instances" on the plater session

Every `file:line` is at OrcaSlicer v2.5.0-beta `345efec64`. The object menu's "Set number of instances…" asks for a count and adds or removes copies to reach it. Setting 0 removes the object. `web-plater-instance-copies.md` deferred both this item and `decrease_instances`' object removal, waiting on object deletion, which `web-plater-delete-object.md` has since ported.

## Upstream boundary
- **The menu item** is "Set number of instances…", "Change the number of instances of the selected object" (`GUI_Factories.cpp:2316-2318`), enabled by `can_increase_instances`: every instance printable (`Plater.cpp:15097-15107`).
- **`Plater::set_number_of_copies`** (`:19386-19406`):
  - `GetNumberFromUser(" ", "Number of copies:", "Copies of the selected object", current count, 0, 1000)` asks for the count; `-1`, a cancel, does nothing.
  - With `diff = num − count`, it calls `increase_instances(diff)` when positive and `decrease_instances(−diff)` when negative.
- **`decrease_instances(num)`** (`:19333-19364`) returns silently unless `can_decrease_instances` (more than one instance, `:15109-15118`).
  - With fewer than `num` instances left over, that is when `num` is at least the count, it calls `Plater::priv::remove(obj_idx)` (`:10727-10747`). That is `Model::delete_object` plus `notify_instance_removed(obj_idx, -1)`, the same model and plate changes as "Delete", without its cut warning.
  - Afterwards it reads `model_object->instances` (`:19354-19355`) to select an instance. When the object was just removed, that pointer is freed: a use-after-free in UI selection code, undefined behaviour, which is not ported.
- **`increase_instances(num)`** (`:19289-19331`) likewise returns silently unless `can_increase_instances`.

## Ares destination
- **`PlaterSession::set_number_of_copies(object, num)`.**
  - `InvalidInput` for no such object, or a count outside the dialog's 0..1000.
  - Otherwise the `diff` dispatch, where a disabled `increase` / `decrease` is a silent no-op, as upstream: a single instance set to 0 stays.
- **`decrease_instances(num)` for `num` at least the count** (and more than one instance) now removes the object through the delete path, replacing the copies slice's `InvalidInput`. The direct `increase_instances` / `decrease_instances` calls keep their `InvalidInput` for the disabled cases: their menu items are disabled there, so only the API reaches them.
- **wasm:** `setNumberOfCopies`.
- **Web:** "Set number of instances…" in the object menu with upstream's enabling rule, opening a "Copies of the selected object" dialog. It has a "Number of copies:" field from 0 to 1000, preset to the current count, plus OK and Cancel.

## Included and deferred
- **Included:** the dialog item, the `diff` dispatch, and `decrease_instances`' object removal.
- **Deferred:** "Fill bed with instances" (`FillBedJob`), and the undo snapshot.

## Oracle and RED
- **The session-edit Native oracle** (`assert_plate_like_native`) on the plater_instances / plater_delete_object fixtures.
- **Cases** in `plater_number_of_instances.rs`:
  1. The Ender-3 cube set to 3: the two copies of the copies slice's case 1, at (121, 121) and (132, 132). It slices like Native.
  2. Set to 3, then to 1: the copies are removed and the cube keeps its place. It slices like Native.
  3. The by-object cubes' two-instance object 0 set to 0: the object is removed as in the delete slice's case 1, and the plate slices like Native.
  4. **Session-only:**
     - a single instance set to 0 stays, returning `Ok`;
     - a count of 1001 returns `InvalidInput`;
     - `decrease_instances(2)` on a two-instance object removes it.
- **Web E2E** `plateNumberOfInstances.spec.ts`: the menu item, dialog value 3, OK, then "Slice plate", compared with the runtime Native oracle for the two-copy export.
- **Expected before GREEN:** cases 1-4 fail on the `todo!()` skeleton of `set_number_of_copies`. The last check of case 4 fails on today's `InvalidInput`.
